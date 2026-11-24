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
