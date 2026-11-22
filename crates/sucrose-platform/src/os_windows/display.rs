//! Monitors and the process DPI awareness.

use std::mem::size_of;

use windows::{
    Win32::{
        Foundation::{LPARAM, RECT},
        Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW},
        UI::HiDpi::{
            AreDpiAwarenessContextsEqual, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, DPI_AWARENESS_PER_MONITOR_AWARE,
            DPI_AWARENESS_SYSTEM_AWARE, DPI_AWARENESS_UNAWARE, GetAwarenessFromDpiAwarenessContext, GetDpiForMonitor,
            GetThreadDpiAwarenessContext, MDT_EFFECTIVE_DPI, SetProcessDpiAwarenessContext,
        },
    },
    core::BOOL,
};

use crate::{DpiAwareness, MonitorInfo, RectPx};

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
    found
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
        rect: rect_px(&info.monitorInfo.rcMonitor),
        work: rect_px(&info.monitorInfo.rcWork),
        dpi: dpi_x,
        primary: info.monitorInfo.dwFlags & PRIMARY_FLAG != 0,
    })
}
