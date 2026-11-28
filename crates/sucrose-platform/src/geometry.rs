//! Pixel geometry shared by every OS implementation. Everything here is physical pixels of the virtual desktop;
//! nothing is ever scaled by DPI.

/// A rectangle in physical pixels of the virtual desktop (never scaled by DPI), edges as in the Win32 `RECT`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RectPx {
    /// Left edge.
    pub left: i32,
    /// Top edge.
    pub top: i32,
    /// Right edge (exclusive).
    pub right: i32,
    /// Bottom edge (exclusive).
    pub bottom: i32,
}

impl RectPx {
    /// Width in pixels.
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    /// Height in pixels.
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    /// This screen rectangle expressed in the client space of a parent window whose client origin sits at
    /// `parent_screen_origin` on the screen. A parent is not guaranteed to start at the virtual-desktop origin.
    pub fn to_parent(&self, parent_screen_origin: (i32, i32)) -> Self {
        let (origin_x, origin_y) = parent_screen_origin;
        Self {
            left: self.left - origin_x,
            top: self.top - origin_y,
            right: self.right - origin_x,
            bottom: self.bottom - origin_y,
        }
    }

    /// The part both rectangles cover; `None` when they do not overlap (right and bottom are exclusive, so touching
    /// edges do not overlap).
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let part = Self {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        };
        (part.left < part.right && part.top < part.bottom).then_some(part)
    }

    /// Whether `other` lies completely inside this rectangle.
    pub fn covers(&self, other: &Self) -> bool {
        self.left <= other.left && self.top <= other.top && self.right >= other.right && self.bottom >= other.bottom
    }
}

/// One display as the OS reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorInfo {
    /// Device name (`\\.\DISPLAY1` on Windows). It names the monitor for the slot map, the topology diff and the
    /// renderers; settings remember a monitor by [`MonitorInfo::stable_id`] instead (D-08).
    pub id: String,
    /// The display path's `monitorDevicePath` (Windows), which carries the connection instance; `None` when the OS
    /// reports none.
    pub device_path: Option<String>,
    /// The EDID key: the manufacturer's three PnP letters, a dash and the product code as four hex digits (for
    /// example `DEL-A0B1`); `None` when the OS did not report valid EDID ids.
    pub edid: Option<String>,
    /// The monitor's friendly name from the OS (EDID), when it reports one.
    pub friendly: Option<String>,
    /// Full monitor rectangle in physical pixels.
    pub rect: RectPx,
    /// Work area (without the taskbar) in physical pixels.
    pub work: RectPx,
    /// Effective DPI of the monitor (96 is 100 %).
    pub dpi: u32,
    /// Whether this is the primary monitor.
    pub primary: bool,
}

impl MonitorInfo {
    /// The id a setting remembers this monitor by (D-08): the display path's device path, or the device name when the
    /// OS reports no path. It is never empty as long as `id` is not.
    pub fn stable_id(&self) -> &str {
        match self.device_path.as_deref() {
            Some(path) if !path.is_empty() => path,
            _ => &self.id,
        }
    }
}

/// What the OS display configuration says about one active display path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayTarget {
    /// The device name of the path's source (`\\.\DISPLAY1`), which joins it to a [`MonitorInfo`].
    pub gdi_name: String,
    /// The target's `monitorDevicePath`; empty when the OS reports none.
    pub device_path: String,
    /// The EDID key (see [`MonitorInfo::edid`]); `None` when the EDID ids are not valid.
    pub edid: Option<String>,
    /// The friendly name of the monitor, when there is one.
    pub friendly: Option<String>,
}

/// The EDID key of a display target: the manufacturer id as three PnP letters, a dash, the product code as four
/// upper-case hex digits. The OS hands the manufacturer id over as the two EDID bytes read little-endian (the first
/// byte, which holds the high bits of the first letter, ends up in the low byte), so the bytes are swapped back first.
/// Returns `None` for an id that does not decode to three letters.
pub fn edid_key(manufacturer: u16, product: u16) -> Option<String> {
    let packed = manufacturer.swap_bytes();
    let letters = [(packed >> 10) & 0x1f, (packed >> 5) & 0x1f, packed & 0x1f];
    if letters.iter().any(|letter| !(1..=26).contains(letter)) {
        return None;
    }
    let mut key: String = letters
        .iter()
        .map(|&letter| char::from(b'A' + (letter as u8) - 1))
        .collect();
    key.push('-');
    key.push_str(&format!("{product:04X}"));
    Some(key)
}

/// Joins the display paths to the monitors by device name (never by position) and returns the monitors in their
/// original order. A monitor without a path, or whose path is empty, keeps `device_path` and `edid` as `None`, so its
/// stable id is its device name. The first target per device name wins, and a device path already given to an earlier
/// monitor is not given again, so no two monitors share a stable id.
pub fn attach_display_ids(monitors: Vec<MonitorInfo>, targets: &[DisplayTarget]) -> Vec<MonitorInfo> {
    let mut taken: Vec<String> = Vec::new();
    monitors
        .into_iter()
        .map(|monitor| {
            let target = targets
                .iter()
                .find(|target| target.gdi_name == monitor.id)
                .filter(|target| !target.device_path.is_empty() && !taken.contains(&target.device_path));
            match target {
                Some(target) => {
                    taken.push(target.device_path.clone());
                    MonitorInfo {
                        device_path: Some(target.device_path.clone()),
                        edid: target.edid.clone(),
                        friendly: target.friendly.clone(),
                        ..monitor
                    }
                }
                None => monitor,
            }
        })
        .collect()
}

/// Where a surface ended up relative to its target and to the window it is a child of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementVerdict {
    /// At its target, and the parent's client area covers all of it.
    Shown,
    /// The surface's actual rectangle differs from the target: the move did not land as asked.
    Misplaced,
    /// At its target, but the parent covers only `visible` of it; the rest is clipped away.
    PartlyHidden {
        /// The part of the target inside the parent's client area.
        visible: RectPx,
    },
    /// At its target, but no part of it lies inside the parent, so nothing of it shows.
    Hidden,
}

/// Judges a placement: `actual` is the surface's screen rectangle read back after moving it to `target`, `parent`
/// the parent's client area on the screen.
pub fn placement_verdict(target: RectPx, actual: RectPx, parent: RectPx) -> PlacementVerdict {
    if actual != target {
        PlacementVerdict::Misplaced
    } else if parent.covers(&target) {
        PlacementVerdict::Shown
    } else {
        parent
            .intersection(&target)
            .map_or(PlacementVerdict::Hidden, |visible| PlacementVerdict::PartlyHidden {
                visible,
            })
    }
}

/// What a move of a desktop surface did, read back from the OS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    /// The screen rectangle that was asked for.
    pub target: RectPx,
    /// The target in the parent's client coordinates, as given to the OS.
    pub local: RectPx,
    /// The parent's client area on the screen; its left/top is the origin `local` was computed from.
    pub parent: RectPx,
    /// The surface's screen rectangle read back.
    pub actual: RectPx,
    /// The whole virtual desktop.
    pub virtual_screen: RectPx,
    /// The verdict on target, actual and parent.
    pub verdict: PlacementVerdict,
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RectPx {
        RectPx {
            left,
            top,
            right,
            bottom,
        }
    }

    fn monitor(id: &str) -> MonitorInfo {
        MonitorInfo {
            id: id.to_owned(),
            device_path: None,
            edid: None,
            friendly: None,
            rect: RectPx::default(),
            work: RectPx::default(),
            dpi: 96,
            primary: false,
        }
    }

    fn target(gdi_name: &str, path: &str, edid: Option<&str>) -> DisplayTarget {
        DisplayTarget {
            gdi_name: gdi_name.to_owned(),
            device_path: path.to_owned(),
            edid: edid.map(str::to_owned),
            friendly: None,
        }
    }

    #[test]
    fn display_ids_are_joined_by_device_name_in_any_order() {
        let targets = [
            target("\\\\.\\DISPLAY9", "path-9", Some("MTT-1337")),
            target("\\\\.\\DISPLAY1", "path-1", Some("DEL-A0B1")),
        ];
        let forward = attach_display_ids(vec![monitor("\\\\.\\DISPLAY1"), monitor("\\\\.\\DISPLAY9")], &targets);
        let backward = attach_display_ids(vec![monitor("\\\\.\\DISPLAY9"), monitor("\\\\.\\DISPLAY1")], &targets);

        assert_eq!(forward[0].stable_id(), "path-1");
        assert_eq!(forward[1].stable_id(), "path-9");
        assert_eq!(backward[0].stable_id(), "path-9");
        assert_eq!(backward[1].stable_id(), "path-1");
        assert_eq!(forward[1].edid.as_deref(), Some("MTT-1337"));
    }

    #[test]
    fn two_monitors_of_one_model_keep_different_stable_ids() {
        let targets = [
            target("\\\\.\\DISPLAY1", "path-a", Some("DEL-A0B1")),
            target("\\\\.\\DISPLAY2", "path-b", Some("DEL-A0B1")),
        ];
        let monitors = attach_display_ids(vec![monitor("\\\\.\\DISPLAY1"), monitor("\\\\.\\DISPLAY2")], &targets);

        assert_eq!(monitors[0].edid, monitors[1].edid);
        assert_ne!(monitors[0].stable_id(), monitors[1].stable_id());
    }

    #[test]
    fn a_device_path_is_never_given_to_two_monitors() {
        let targets = [
            target("\\\\.\\DISPLAY1", "same-path", None),
            target("\\\\.\\DISPLAY2", "same-path", None),
        ];
        let monitors = attach_display_ids(vec![monitor("\\\\.\\DISPLAY1"), monitor("\\\\.\\DISPLAY2")], &targets);

        assert_eq!(monitors[0].stable_id(), "same-path");
        assert_eq!(monitors[1].stable_id(), "\\\\.\\DISPLAY2");
    }

    #[test]
    fn a_monitor_without_a_path_falls_back_to_its_device_name() {
        let targets = [
            target("\\\\.\\DISPLAY1", "", Some("DEL-A0B1")),
            target("\\\\.\\DISPLAY3", "path-3", None),
        ];
        let monitors = attach_display_ids(vec![monitor("\\\\.\\DISPLAY1"), monitor("\\\\.\\DISPLAY2")], &targets);

        assert_eq!(monitors[0].stable_id(), "\\\\.\\DISPLAY1");
        assert_eq!(monitors[0].edid, None);
        assert_eq!(monitors[1].stable_id(), "\\\\.\\DISPLAY2");
        assert_eq!(
            attach_display_ids(vec![monitor("\\\\.\\DISPLAY1")], &[])[0].stable_id(),
            "\\\\.\\DISPLAY1"
        );
    }

    #[test]
    fn an_empty_monitor_list_stays_empty() {
        assert!(attach_display_ids(Vec::new(), &[target("\\\\.\\DISPLAY1", "path-1", None)]).is_empty());
    }

    #[test]
    fn an_empty_device_path_on_the_monitor_is_not_a_stable_id() {
        let mut empty = monitor("\\\\.\\DISPLAY1");
        empty.device_path = Some(String::new());
        assert_eq!(empty.stable_id(), "\\\\.\\DISPLAY1");
    }

    #[test]
    fn edid_key_decodes_the_manufacturer_letters_and_the_product_code() {
        // Dell: EDID bytes 0x10 0xAC, read little-endian by the OS as 0xAC10.
        assert_eq!(edid_key(0xAC10, 0xA0B1).as_deref(), Some("DEL-A0B1"));
        assert_eq!(edid_key(0xAC10, 0x0007).as_deref(), Some("DEL-0007"));
    }

    #[test]
    fn edid_key_refuses_an_id_that_is_not_three_letters() {
        assert_eq!(edid_key(0, 1), None);
        assert_eq!(edid_key(0xFFFF, 1), None);
    }

    /// Progman's rectangle in layout B of the UAT run: it spans both monitors.
    const PARENT_B: RectPx = rect(-1920, 0, 1920, 1080);

    #[test]
    fn placement_verdict_uat_layout_c_is_hidden() {
        let target = rect(1920, 0, 3840, 1080);
        assert_eq!(placement_verdict(target, target, PARENT_B), PlacementVerdict::Hidden);
    }

    #[test]
    fn placement_verdict_inside_the_parent_is_shown() {
        let target = rect(0, 0, 1920, 1080);
        assert_eq!(placement_verdict(target, target, PARENT_B), PlacementVerdict::Shown);
    }

    #[test]
    fn placement_verdict_half_outside_the_parent_is_partly_hidden() {
        let target = rect(960, 0, 2880, 1080);
        assert_eq!(
            placement_verdict(target, target, PARENT_B),
            PlacementVerdict::PartlyHidden {
                visible: rect(960, 0, 1920, 1080)
            }
        );
    }

    #[test]
    fn placement_verdict_a_wrong_actual_rectangle_is_misplaced() {
        let target = rect(0, 0, 1920, 1080);
        let actual = rect(0, 0, 1280, 720);
        assert_eq!(placement_verdict(target, actual, PARENT_B), PlacementVerdict::Misplaced);
    }

    #[test]
    fn placement_verdict_touching_edges_do_not_overlap() {
        let target = rect(1920, 0, 3840, 1080);
        let parent = rect(0, 0, 1920, 1080);
        assert_eq!(placement_verdict(target, target, parent), PlacementVerdict::Hidden);
    }

    #[test]
    fn placement_intersection_of_overlapping_rects_is_their_common_part() {
        let a = rect(0, 0, 100, 100);
        let b = rect(50, 25, 150, 75);
        assert_eq!(a.intersection(&b), Some(rect(50, 25, 100, 75)));
        assert_eq!(b.intersection(&a), Some(rect(50, 25, 100, 75)));
    }

    #[test]
    fn placement_intersection_of_touching_or_disjoint_rects_is_none() {
        let a = rect(0, 0, 100, 100);
        assert_eq!(a.intersection(&rect(100, 0, 200, 100)), None);
        assert_eq!(a.intersection(&rect(0, 100, 100, 200)), None);
        assert_eq!(a.intersection(&rect(300, 300, 400, 400)), None);
    }

    #[test]
    fn placement_intersection_of_a_contained_rect_is_itself() {
        let outer = rect(0, 0, 100, 100);
        let inner = rect(10, 10, 20, 20);
        assert_eq!(outer.intersection(&inner), Some(inner));
        assert_eq!(inner.intersection(&outer), Some(inner));
    }
}
