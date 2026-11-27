//! The stretch rectangles of the built-in renderer: which part of a frame (`src`) is drawn onto which part of the surface
//! (`dest`) for each [`Stretch`] mode (legacy mapping, `docs/legacy/engines.md` §3: a WPF `Image` with `Stretch` set to the
//! enum). Pure code, shared by every presenter so that Direct2D and GDI draw the same picture.

use sucrose_core::settings::Stretch;

use crate::decode::Scaling;

/// A rectangle as left, top, right, bottom in pixels.
pub type Rect = [f32; 4];

/// How the decoder has to prepare a frame for `stretch`, so a frame never holds more pixels than the surface can show
/// (T-04-66): Fill and Uniform need the whole frame, scaled down to fit; UniformToFill needs the whole frame scaled down
/// only as far as it still covers the surface; None shows the centre of the frame at 1:1, so only that part is kept.
#[must_use]
pub fn scaling(stretch: Stretch) -> Scaling {
    match stretch {
        Stretch::Fill | Stretch::Uniform => Scaling::Within,
        Stretch::UniformToFill => Scaling::Cover,
        Stretch::None => Scaling::Crop,
    }
}

/// The rectangles that draw a `source` sized frame onto a `surface` sized window for `stretch`: the destination on the
/// surface and the part of the frame that goes there. Both lie inside their bounds; the destination is smaller than the
/// surface when the frame is letterboxed (None and Uniform) and the source is smaller than the frame when it is cropped
/// (None and UniformToFill). With the same aspect ratio Uniform, UniformToFill and Fill all give the whole of both.
#[must_use]
pub fn fit(stretch: Stretch, source: (u32, u32), surface: (u32, u32)) -> (Rect, Rect) {
    let (width, height) = (f64::from(source.0.max(1)), f64::from(source.1.max(1)));
    let (area_w, area_h) = (f64::from(surface.0.max(1)), f64::from(surface.1.max(1)));
    let whole = (
        [0.0, 0.0, area_w as f32, area_h as f32],
        [0.0, 0.0, width as f32, height as f32],
    );
    let same_aspect = u64::from(source.0.max(1)) * u64::from(surface.1.max(1))
        == u64::from(source.1.max(1)) * u64::from(surface.0.max(1));
    let scale = match stretch {
        Stretch::Fill => return whole,
        Stretch::Uniform | Stretch::UniformToFill if same_aspect => return whole,
        Stretch::None => 1.0,
        Stretch::Uniform => (area_w / width).min(area_h / height),
        Stretch::UniformToFill => (area_w / width).max(area_h / height),
    };
    // The frame scaled and centred on the surface, then clipped to the surface: what is clipped is not in the source.
    let (scaled_w, scaled_h) = (width * scale, height * scale);
    let (left, top) = ((area_w - scaled_w) / 2.0, (area_h - scaled_h) / 2.0);
    let (dest_left, dest_top) = (left.max(0.0), top.max(0.0));
    let (dest_right, dest_bottom) = ((left + scaled_w).min(area_w), (top + scaled_h).min(area_h));
    let to_source = |value: f64, origin: f64| ((value - origin) / scale).clamp(0.0, f64::MAX);
    let src = [
        to_source(dest_left, left).min(width),
        to_source(dest_top, top).min(height),
        to_source(dest_right, left).min(width),
        to_source(dest_bottom, top).min(height),
    ];
    (
        [dest_left as f32, dest_top as f32, dest_right as f32, dest_bottom as f32],
        src.map(|value| value as f32),
    )
}

/// A rectangle as whole pixels (x, y, width, height), each side at least one pixel, for the GDI presenter.
#[must_use]
pub fn pixels(rect: Rect) -> (i32, i32, i32, i32) {
    let [left, top, right, bottom] = rect.map(f32::round);
    let (x, y) = (left as i32, top as i32);
    (x, y, ((right as i32) - x).max(1), ((bottom as i32) - y).max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SURFACE: (u32, u32) = (1920, 1080);

    #[test]
    fn none_centres_a_small_frame_at_one_to_one() {
        let (dest, src) = fit(Stretch::None, (320, 180), SURFACE);
        assert_eq!(dest, [800.0, 450.0, 1120.0, 630.0]);
        assert_eq!(src, [0.0, 0.0, 320.0, 180.0]);
    }

    #[test]
    fn none_crops_a_large_frame_to_its_centre() {
        let (dest, src) = fit(Stretch::None, (3840, 2160), SURFACE);
        assert_eq!(dest, [0.0, 0.0, 1920.0, 1080.0]);
        assert_eq!(src, [960.0, 540.0, 2880.0, 1620.0]);
    }

    #[test]
    fn uniform_letterboxes_and_uniform_to_fill_crops() {
        let (dest, src) = fit(Stretch::Uniform, (1000, 1000), SURFACE);
        assert_eq!(dest, [420.0, 0.0, 1500.0, 1080.0]);
        assert_eq!(src, [0.0, 0.0, 1000.0, 1000.0]);

        let (dest, src) = fit(Stretch::UniformToFill, (1000, 1000), SURFACE);
        assert_eq!(dest, [0.0, 0.0, 1920.0, 1080.0]);
        assert_eq!(src, [0.0, 218.75, 1000.0, 781.25]);
    }

    #[test]
    fn fill_is_the_whole_surface_and_the_whole_frame() {
        let (dest, src) = fit(Stretch::Fill, (1000, 1000), SURFACE);
        assert_eq!(dest, [0.0, 0.0, 1920.0, 1080.0]);
        assert_eq!(src, [0.0, 0.0, 1000.0, 1000.0]);
    }

    #[test]
    fn the_same_aspect_ratio_gives_the_same_rectangles_for_every_stretching_mode() {
        for source in [(320, 180), (1920, 1080), (3840, 2160), (7, 4)] {
            let surface = if source == (7, 4) { (1400, 800) } else { SURFACE };
            let fill = fit(Stretch::Fill, source, surface);
            assert_eq!(fit(Stretch::Uniform, source, surface), fill, "{source:?}");
            assert_eq!(fit(Stretch::UniformToFill, source, surface), fill, "{source:?}");
        }
    }

    #[test]
    fn a_tall_frame_on_a_wide_surface_leaves_bars_left_and_right_for_uniform() {
        let (dest, src) = fit(Stretch::Uniform, (540, 1080), SURFACE);
        assert_eq!(dest, [690.0, 0.0, 1230.0, 1080.0]);
        assert_eq!(src, [0.0, 0.0, 540.0, 1080.0]);
    }

    #[test]
    fn a_degenerate_size_never_panics_and_stays_inside_the_surface() {
        for stretch in Stretch::ALL {
            let (dest, src) = fit(*stretch, (0, 0), (0, 0));
            assert!(
                dest.iter().all(|value| value.is_finite() && *value >= 0.0),
                "{stretch:?}"
            );
            assert!(
                src.iter().all(|value| value.is_finite() && *value >= 0.0),
                "{stretch:?}"
            );
        }
    }

    #[test]
    fn the_destination_and_the_source_stay_inside_their_bounds() {
        for stretch in Stretch::ALL {
            for source in [(1, 1), (100, 30), (30, 100), (4000, 3000), (1920, 1080)] {
                let (dest, src) = fit(*stretch, source, SURFACE);
                assert!(dest[0] >= 0.0 && dest[1] >= 0.0 && dest[2] <= 1920.0 && dest[3] <= 1080.0);
                assert!(src[0] >= 0.0 && src[1] >= 0.0 && src[2] <= source.0 as f32 && src[3] <= source.1 as f32);
                assert!(dest[2] > dest[0] && dest[3] > dest[1] && src[2] > src[0] && src[3] > src[1]);
            }
        }
    }

    #[test]
    fn pixels_are_whole_and_at_least_one_pixel_wide() {
        assert_eq!(pixels([420.0, 0.0, 1500.0, 1080.0]), (420, 0, 1080, 1080));
        assert_eq!(pixels([0.0, 218.75, 1000.0, 781.25]), (0, 219, 1000, 562));
        assert_eq!(pixels([5.0, 5.0, 5.0, 5.0]), (5, 5, 1, 1));
    }

    #[test]
    fn the_decoder_keeps_what_each_mode_can_show() {
        assert_eq!(scaling(Stretch::Fill), Scaling::Within);
        assert_eq!(scaling(Stretch::Uniform), Scaling::Within);
        assert_eq!(scaling(Stretch::UniformToFill), Scaling::Cover);
        assert_eq!(scaling(Stretch::None), Scaling::Crop);
    }
}
