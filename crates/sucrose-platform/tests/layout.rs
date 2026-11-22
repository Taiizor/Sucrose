//! The attach decision and the rect math, tested on every OS against recorded shell trees.

use sucrose_platform::{
    RectPx,
    layout::{DesktopLayout, RaisedOrder, ShellSnapshot, SiblingKind, WindowNode, classify, raised_order},
};

const WS_EX_NOREDIRECTIONBITMAP: u32 = 0x0020_0000;

fn snapshot(json: &str) -> ShellSnapshot {
    serde_json::from_str(json).expect("the fixture is a valid snapshot")
}

fn node(class: &str, ex_style: u32) -> WindowNode {
    WindowNode {
        class: class.to_owned(),
        ex_style,
        visible: true,
        rect: [0, 0, 1920, 1080],
        ..WindowNode::default()
    }
}

#[test]
fn the_26300_probe_is_raised() {
    let tree = snapshot(include_str!("fixtures/desktop-26300.json"));
    assert_eq!(classify(&tree), DesktopLayout::Raised);
}

#[test]
fn the_classic_tree_is_classic() {
    let tree = snapshot(include_str!("fixtures/desktop-classic.json"));
    assert_eq!(classify(&tree), DesktopLayout::Classic);
}

#[test]
fn without_progman_the_layout_is_unsupported() {
    let tree = ShellSnapshot {
        progman: None,
        top_level_with_defview: vec![node("WorkerW", 0)],
    };
    assert_eq!(classify(&tree), DesktopLayout::Unsupported);
}

#[test]
fn the_no_redirection_bitmap_flag_alone_decides_raised() {
    let tree = ShellSnapshot {
        progman: Some(node("Progman", WS_EX_NOREDIRECTIONBITMAP)),
        top_level_with_defview: Vec::new(),
    };
    assert_eq!(classify(&tree), DesktopLayout::Raised);
}

#[test]
fn a_plain_progman_with_nothing_holding_the_icons_is_unsupported() {
    let tree = ShellSnapshot {
        progman: Some(node("Progman", 0x80)),
        top_level_with_defview: Vec::new(),
    };
    assert_eq!(classify(&tree), DesktopLayout::Unsupported);
}

#[test]
fn to_parent_subtracts_a_negative_parent_origin() {
    let rect = RectPx {
        left: 100,
        top: 0,
        right: 2020,
        bottom: 1080,
    };
    let mapped = rect.to_parent((-1920, 0));
    assert_eq!(
        mapped,
        RectPx {
            left: 2020,
            top: 0,
            right: 3940,
            bottom: 1080,
        }
    );
}

#[test]
fn to_parent_with_a_zero_origin_changes_nothing() {
    let rect = RectPx {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    };
    assert_eq!(rect.to_parent((0, 0)), rect);
}

#[test]
fn rects_stay_in_physical_pixels_at_200_percent() {
    // A 3840x2160 monitor at 200 % scale is still 3840 wide in the rect: no logical scaling anywhere.
    let monitor = RectPx {
        left: 0,
        top: 0,
        right: 3840,
        bottom: 2160,
    };
    let mapped = monitor.to_parent((-3840, 0));
    assert_eq!((mapped.width(), mapped.height()), (3840, 2160));
}

#[test]
fn covers_means_completely_inside() {
    let outer = RectPx {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    };
    let inner = RectPx {
        left: 10,
        top: 10,
        right: 100,
        bottom: 100,
    };
    let overhang = RectPx {
        left: 0,
        top: 0,
        right: 1921,
        bottom: 1080,
    };
    assert!(outer.covers(&outer));
    assert!(outer.covers(&inner));
    assert!(!outer.covers(&overhang));
    assert!(!inner.covers(&outer));
}

mod raised_order_cases {
    use super::{RaisedOrder, SiblingKind::*, raised_order};

    #[test]
    fn raised_order_icons_surface_wallpaper_is_correct() {
        assert_eq!(raised_order(&[Icons, Surface, Wallpaper]), RaisedOrder::Correct);
    }

    #[test]
    fn raised_order_wallpaper_above_the_surface_is_the_uat_failure() {
        assert_eq!(
            raised_order(&[Icons, Wallpaper, Surface]),
            RaisedOrder::WallpaperAboveSurface
        );
    }

    #[test]
    fn raised_order_surface_above_the_icons_is_rejected() {
        assert_eq!(
            raised_order(&[Surface, Icons, Wallpaper]),
            RaisedOrder::SurfaceAboveIcons
        );
    }

    #[test]
    fn raised_order_without_a_wallpaper_yet_is_correct() {
        assert_eq!(raised_order(&[Icons, Surface]), RaisedOrder::Correct);
    }

    #[test]
    fn raised_order_another_renderers_surface_is_just_a_sibling() {
        assert_eq!(raised_order(&[Icons, Other, Surface, Wallpaper]), RaisedOrder::Correct);
    }

    #[test]
    fn raised_order_reports_a_missing_surface_or_icons() {
        assert_eq!(raised_order(&[Icons, Wallpaper]), RaisedOrder::SurfaceMissing);
        assert_eq!(raised_order(&[Surface, Wallpaper]), RaisedOrder::IconsMissing);
    }
}
