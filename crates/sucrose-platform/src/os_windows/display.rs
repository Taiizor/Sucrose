//! Monitors and the process DPI awareness.

use std::mem::size_of;

use windows::{
    Win32::{
        Devices::Display::{
            DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
            DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_SOURCE_DEVICE_NAME,
            DISPLAYCONFIG_TARGET_DEVICE_NAME, DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes,
            QDC_ONLY_ACTIVE_PATHS, QueryDisplayConfig,
        },
        Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, LPARAM, RECT},
        Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW},
        UI::HiDpi::{
            AreDpiAwarenessContextsEqual, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, DPI_AWARENESS_PER_MONITOR_AWARE,
            DPI_AWARENESS_SYSTEM_AWARE, DPI_AWARENESS_UNAWARE, GetAwarenessFromDpiAwarenessContext, GetDpiForMonitor,
            GetThreadDpiAwarenessContext, MDT_EFFECTIVE_DPI, SetProcessDpiAwarenessContext,
        },
    },
    core::BOOL,
};

use crate::{DisplayTarget, DpiAwareness, MonitorInfo, RectPx, attach_display_ids, edid_key};

/// `MONITORINFOF_PRIMARY`.
const PRIMARY_FLAG: u32 = 1;

fn rect_px(rect: &RECT) -> RectPx {
    RectPx {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}

/// Asks for Per-Monitor-V2 and returns the awareness the calling thread actually has afterwards.
pub(crate) fn init_dpi_awareness() -> crate::DpiAwareness {
    // SAFETY: no pointers are involved; the call only changes this process's default DPI context. It fails when the
    // context was already set (for example by a manifest), which is fine because the effective value is read below.
    if let Err(error) = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) } {
        tracing::debug!(%error, "the process DPI awareness was already set");
    }
    // SAFETY: both calls only read the calling thread's DPI context.
    let context = unsafe { GetThreadDpiAwarenessContext() };
    // SAFETY: both arguments are valid DPI contexts (one from the OS, one a documented constant).
    if unsafe { AreDpiAwarenessContextsEqual(context, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) }.as_bool() {
        return DpiAwareness::PerMonitorV2;
    }
    // SAFETY: `context` came from GetThreadDpiAwarenessContext.
    match unsafe { GetAwarenessFromDpiAwarenessContext(context) } {
        DPI_AWARENESS_PER_MONITOR_AWARE => DpiAwareness::PerMonitor,
        DPI_AWARENESS_SYSTEM_AWARE => DpiAwareness::System,
        DPI_AWARENESS_UNAWARE => DpiAwareness::Unaware,
        _ => DpiAwareness::Unknown,
    }
}

/// The connected monitors in physical pixels, the primary one first.
pub(crate) fn monitors() -> Vec<MonitorInfo> {
    unsafe extern "system" fn collect(monitor: HMONITOR, _dc: HDC, _rect: *mut RECT, data: LPARAM) -> BOOL {
        // SAFETY: `data` is the address of the Vec that `monitors` passed to EnumDisplayMonitors and keeps alive for
        // the whole (synchronous) enumeration; no other reference to it exists meanwhile.
        let list = unsafe { &mut *(data.0 as *mut Vec<MonitorInfo>) };
        if let Some(info) = monitor_info(monitor) {
            list.push(info);
        }
        true.into()
    }

    let mut found: Vec<MonitorInfo> = Vec::new();
    // SAFETY: the callback only touches the Vec behind the pointer passed here, which outlives the call.
    let _ = unsafe { EnumDisplayMonitors(None, None, Some(collect), LPARAM(&raw mut found as isize)) };
    found.sort_by_key(|monitor| !monitor.primary);
    attach_display_ids(found, &display_targets())
}

/// The text of a fixed-size UTF-16 buffer up to its first NUL (or its whole length when it has none).
fn wide_text(buffer: &[u16]) -> String {
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}

/// The active display paths with their device path, EDID key and friendly name (D-08). A failing call returns what was
/// read so far: the monitors then fall back to their device name.
pub(crate) fn display_targets() -> Vec<DisplayTarget> {
    let Some(paths) = active_paths() else {
        return Vec::new();
    };
    let mut targets = Vec::with_capacity(paths.len());
    for path in &paths {
        let mut source = DISPLAYCONFIG_SOURCE_DEVICE_NAME::default();
        source.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
        source.header.size = size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
        source.header.adapterId = path.sourceInfo.adapterId;
        source.header.id = path.sourceInfo.id;
        // SAFETY: `source` is a DISPLAYCONFIG_SOURCE_DEVICE_NAME whose header carries its type, its size and the
        // path's source ids; the API writes at most `size` bytes into it.
        let status = unsafe { DisplayConfigGetDeviceInfo(&raw mut source.header) };
        if status != 0 {
            tracing::debug!(status, "the display configuration has no source name for a path");
            return targets;
        }

        let mut target = DISPLAYCONFIG_TARGET_DEVICE_NAME::default();
        target.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
        target.header.size = size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
        target.header.adapterId = path.targetInfo.adapterId;
        target.header.id = path.targetInfo.id;
        // SAFETY: `target` is a DISPLAYCONFIG_TARGET_DEVICE_NAME whose header carries its type, its size and the
        // path's target ids; the API writes at most `size` bytes into it.
        let status = unsafe { DisplayConfigGetDeviceInfo(&raw mut target.header) };
        if status != 0 {
            tracing::debug!(status, "the display configuration has no target name for a path");
            return targets;
        }

        // SAFETY: both union views are plain integers of the same size; reading the value view is always valid.
        let edid_valid = unsafe { target.flags.Anonymous.value } & EDID_IDS_VALID != 0;
        let friendly = wide_text(&target.monitorFriendlyDeviceName);
        targets.push(DisplayTarget {
            gdi_name: wide_text(&source.viewGdiDeviceName),
            device_path: wide_text(&target.monitorDevicePath),
            edid: if edid_valid {
                edid_key(target.edidManufactureId, target.edidProductCodeId)
            } else {
                None
            },
            friendly: (!friendly.is_empty()).then_some(friendly),
        });
    }
    targets
}

/// `DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS.edidIdsValid`: the manufacturer and product ids are valid.
const EDID_IDS_VALID: u32 = 0b100;

/// The active display paths, or `None` when the OS cannot say. The path count can change between asking for the buffer
/// sizes and reading, so a too-small buffer is asked again once.
fn active_paths() -> Option<Vec<DISPLAYCONFIG_PATH_INFO>> {
    for attempt in 0..2 {
        let (mut path_count, mut mode_count) = (0_u32, 0_u32);
        // SAFETY: both out pointers are valid for the call.
        let status = unsafe { GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count) };
        if status != ERROR_SUCCESS {
            tracing::debug!(status = status.0, "the display configuration size is not available");
            return None;
        }
        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];
        // SAFETY: `paths` and `modes` hold exactly the element counts passed in `path_count` and `mode_count`, which the
        // call lowers to what it filled in; the topology out pointer is optional and left out.
        let status = unsafe {
            QueryDisplayConfig(
                QDC_ONLY_ACTIVE_PATHS,
                &mut path_count,
                paths.as_mut_ptr(),
                &mut mode_count,
                modes.as_mut_ptr(),
                None,
            )
        };
        if status == ERROR_SUCCESS {
            paths.truncate(path_count as usize);
            return Some(paths);
        }
        if status != ERROR_INSUFFICIENT_BUFFER || attempt == 1 {
            tracing::debug!(status = status.0, "the display configuration is not available");
            return None;
        }
    }
    None
}

fn monitor_info(monitor: HMONITOR) -> Option<MonitorInfo> {
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
    // SAFETY: `info` is a MONITORINFOEXW whose cbSize is set; the API takes it through its MONITORINFO prefix.
    let ok = unsafe { GetMonitorInfoW(monitor, (&raw mut info).cast::<MONITORINFO>()) };
    if !ok.as_bool() {
        return None;
    }
    let (mut dpi_x, mut dpi_y) = (96_u32, 96_u32);
    // SAFETY: both out pointers are valid. A failure leaves the 100 % default.
    let _ = unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) };
    let name_len = info
        .szDevice
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(info.szDevice.len());
    Some(MonitorInfo {
        id: String::from_utf16_lossy(&info.szDevice[..name_len]),
        device_path: None,
        edid: None,
        friendly: None,
        rect: rect_px(&info.monitorInfo.rcMonitor),
        work: rect_px(&info.monitorInfo.rcWork),
        dpi: dpi_x,
        primary: info.monitorInfo.dwFlags & PRIMARY_FLAG != 0,
    })
}
