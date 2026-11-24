//! The grid arithmetic of the library page (LIB-04): columns, card width and page slices. Pure numbers, no GPUI.

use std::ops::Range;

/// Narrowest card; the column count follows from it (legacy parity).
pub const MIN_CARD_WIDTH: f32 = 260.0;
/// Widest card; a row of capped cards is centered.
pub const MAX_CARD_WIDTH: f32 = 400.0;
/// Height of every card.
pub const CARD_HEIGHT: f32 = 160.0;

/// The number of columns for a content `width` and a `gap`: as many cards of at least [`MIN_CARD_WIDTH`] as fit, at least
/// one, and at most `max_per_row` when that is above zero (the `AdaptiveLayout` setting).
pub fn columns(width: f32, gap: f32, max_per_row: usize) -> usize {
    let fit = ((width + gap) / (MIN_CARD_WIDTH + gap)).floor().max(1.0) as usize;
    if max_per_row > 0 { fit.min(max_per_row) } else { fit }
}

/// The width of one card when `columns` of them share `width` with `gap` between them, capped at [`MAX_CARD_WIDTH`].
pub fn card_width(width: f32, gap: f32, columns: usize) -> f32 {
    let columns = columns.max(1) as f32;
    ((width - gap * (columns - 1.0)) / columns).min(MAX_CARD_WIDTH)
}

/// The width a row of `columns` cards of `card_width` takes, gaps included.
pub fn row_width(card_width: f32, gap: f32, columns: usize) -> f32 {
    let columns = columns.max(1) as f32;
    card_width * columns + gap * (columns - 1.0)
}

/// How many pages `count` themes make with `size` themes on a page.
pub fn page_count(count: usize, size: usize) -> usize {
    if size == 0 { 0 } else { count.div_ceil(size) }
}

/// The themes of 1-based `page`, as a range of indexes into the sorted list.
pub fn page_slice(count: usize, page: usize, size: usize) -> Range<usize> {
    let start = ((page - 1) * size).min(count);
    start..(start + size).min(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GAPS: [f32; 4] = [0.0, 5.0, 10.0, 25.0];

    #[test]
    fn layout_columns_follow_the_legacy_formula() {
        assert_eq!(columns(900.0 - 32.0, 5.0, 0), 3, "3 columns at the 900 px minimum");
        assert_eq!(columns(2000.0, 5.0, 0), 7);
        assert_eq!(columns(100.0, 5.0, 0), 1, "never fewer than one");
        assert_eq!(columns(0.0, 5.0, 0), 1);
        assert_eq!(columns(-50.0, 5.0, 0), 1);
        assert_eq!(columns(f32::NAN, 5.0, 0), 1);
    }

    #[test]
    fn layout_columns_change_exactly_at_the_card_boundary() {
        for gap in GAPS {
            for n in 1..=8_usize {
                let exact = MIN_CARD_WIDTH * n as f32 + gap * (n - 1) as f32;
                assert_eq!(columns(exact, gap, 0), n, "n={n} gap={gap}");
                if n > 1 {
                    assert_eq!(columns(exact - 1.0, gap, 0), n - 1, "one pixel less, n={n} gap={gap}");
                }
            }
        }
    }

    #[test]
    fn layout_columns_absorb_float_noise_at_the_boundary() {
        // A width that a scale factor left a hair below the boundary still reaches it.
        assert_eq!(columns(1055.0 - 0.0005, 5.0, 0), 4);
        assert_eq!(columns(1055.0 - 0.5, 5.0, 0), 3);
    }

    #[test]
    fn layout_columns_are_capped_by_the_adaptive_layout_setting() {
        assert_eq!(columns(2000.0, 5.0, 2), 2);
        assert_eq!(columns(2000.0, 5.0, 100), 7, "a cap above the fit changes nothing");
        assert_eq!(columns(300.0, 5.0, 4), 1);
    }

    #[test]
    fn layout_card_width_is_capped_and_a_capped_row_is_narrower_than_the_content() {
        assert_eq!(card_width(1000.0, 5.0, 1), MAX_CARD_WIDTH);
        assert!(row_width(card_width(1000.0, 5.0, 1), 5.0, 1) < 1000.0);
        assert_eq!(card_width(868.0, 5.0, 3), (868.0 - 10.0) / 3.0);
    }

    #[test]
    fn layout_card_width_is_never_negative() {
        assert_eq!(card_width(-20.0, 5.0, 1), 0.0);
        assert_eq!(card_width(10.0, 25.0, 3), 0.0);
    }

    #[test]
    fn layout_a_row_never_exceeds_the_content_width() {
        for gap in GAPS {
            let mut width = 100.0_f32;
            while width < 3000.0 {
                for cap in [0, 1, 2, 3, 5] {
                    let n = columns(width, gap, cap);
                    let card = card_width(width, gap, n);
                    assert!(card <= MAX_CARD_WIDTH);
                    assert!(
                        row_width(card, gap, n) <= width.max(0.0) || card == 0.0,
                        "width={width} gap={gap} cap={cap} n={n} card={card}"
                    );
                }
                width += 37.3;
            }
        }
    }

    #[test]
    fn layout_page_count_rounds_up() {
        assert_eq!(page_count(0, 30), 0);
        assert_eq!(page_count(30, 30), 1);
        assert_eq!(page_count(31, 30), 2);
        assert_eq!(page_count(5, 0), 0);
    }

    #[test]
    fn layout_page_slice_is_the_themes_of_a_page() {
        assert_eq!(page_slice(31, 1, 30), 0..30);
        assert_eq!(page_slice(31, 2, 30), 30..31);
        assert_eq!(page_slice(0, 1, 30), 0..0);
    }

    #[test]
    fn layout_page_slice_survives_pages_out_of_range() {
        assert_eq!(page_slice(31, 0, 30), 0..30, "page 0 is the first page");
        assert_eq!(page_slice(31, 3, 30), 31..31, "past the last page is empty");
        assert_eq!(page_slice(31, usize::MAX, 30), 31..31);
        assert_eq!(page_slice(31, 1, 0), 0..0);
    }
}
