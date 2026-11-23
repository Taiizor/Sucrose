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
    /// Stable device name (`\\.\DISPLAY1` on Windows).
    pub id: String,
    /// Full monitor rectangle in physical pixels.
    pub rect: RectPx,
    /// Work area (without the taskbar) in physical pixels.
    pub work: RectPx,
    /// Effective DPI of the monitor (96 is 100 %).
    pub dpi: u32,
    /// Whether this is the primary monitor.
    pub primary: bool,
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
