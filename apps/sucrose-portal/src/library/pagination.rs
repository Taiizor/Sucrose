//! The pagination bar of the library page (LIB-04, UI-SPEC E5): the page buttons follow the legacy control's scheme and
//! the jump field clamps what the user types. The bar is composed from Buttons and an Input because gpui-component's
//! Pagination has no jump field and no fixed nine-slot scheme (RESEARCH Pattern 7).

use std::{cell::Cell, rc::Rc};

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    Disableable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::i18n::tr;

/// Height of the bar (UI-SPEC spacing: 2xl).
pub const BAR_HEIGHT: f32 = 48.0;

/// More pages than this use the ellipsis scheme.
const FULL_ROW_PAGES: usize = 6;

/// One slot of the page row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageButton {
    /// A page number button.
    Page(usize),
    /// A disabled "..." slot where pages are skipped.
    Ellipsis,
}

/// The page row for `current` of `total` pages, in the legacy control's scheme: every page up to six pages; above that
/// nine slots, the first two and last two pages always shown, with the pages around the current one or the first five or
/// last five, and "..." slots where pages are skipped. No row (the bar is hidden) at one page or fewer.
pub fn page_buttons(current: usize, total: usize) -> Vec<PageButton> {
    use PageButton::{Ellipsis, Page};
    if total <= 1 {
        return Vec::new();
    }
    if total <= FULL_ROW_PAGES {
        return (1..=total).map(Page).collect();
    }
    let current = current.clamp(1, total);
    if current <= 4 {
        vec![
            Page(1),
            Page(2),
            Page(3),
            Page(4),
            Page(5),
            Ellipsis,
            Ellipsis,
            Page(total - 1),
            Page(total),
        ]
    } else if total - current < 4 {
        vec![
            Page(1),
            Page(2),
            Ellipsis,
            Ellipsis,
            Page(total - 4),
            Page(total - 3),
            Page(total - 2),
            Page(total - 1),
            Page(total),
        ]
    } else {
        vec![
            Page(1),
            Page(2),
            Ellipsis,
            Page(current - 1),
            Page(current),
            Page(current + 1),
            Ellipsis,
            Page(total - 1),
            Page(total),
        ]
    }
}

/// The page a jump field text asks for, clamped to 1..=`total`; `None` for an empty or non-numeric text.
pub fn clamp_jump(text: &str, total: usize) -> Option<usize> {
    let text = text.trim();
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let last = total.max(1);
    // A number too large for `usize` is beyond the last page like any other.
    let page = text.parse::<usize>().unwrap_or(usize::MAX);
    Some(page.clamp(1, last))
}

/// How many digits the last page number has.
fn digits(total: usize) -> usize {
    total.max(1).to_string().len()
}

/// Whether the jump field may hold `text`: digits only, at most as many as the last page number has.
pub fn jump_text_allowed(text: &str, total: usize) -> bool {
    text.len() <= digits(total) && text.bytes().all(|byte| byte.is_ascii_digit())
}

/// A handler that gets a page number.
pub type PageHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// A handler for the Jump button.
pub type JumpHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// The state of the jump field: it takes only what `total` (kept current by the view) allows.
pub fn jump_state(
    total: &Rc<Cell<usize>>,
    window: &mut Window,
    cx: &mut Context<super::LibraryView>,
) -> Entity<InputState> {
    let total = Rc::clone(total);
    cx.new(|cx| InputState::new(window, cx).validate(move |text, _| jump_text_allowed(text, total.get())))
}

/// The bar: Previous, the page row, Next, the numeric field and Jump. `on_page` gets the page a button asks for and
/// `on_jump` runs for the Jump button.
pub fn bar(
    current: usize,
    total: usize,
    jump: &Entity<InputState>,
    on_page: PageHandler,
    on_jump: JumpHandler,
) -> AnyElement {
    let previous = {
        let on_page = Rc::clone(&on_page);
        Button::new("page-previous")
            .ghost()
            .icon(IconName::ChevronLeft)
            .accessibility_label(tr("Portal.Pagination.Previous"))
            .disabled(current <= 1)
            .on_click(move |_, window, cx| on_page(current.saturating_sub(1).max(1), window, cx))
    };
    let next = {
        let on_page = Rc::clone(&on_page);
        Button::new("page-next")
            .ghost()
            .icon(IconName::ChevronRight)
            .accessibility_label(tr("Portal.Pagination.Next"))
            .disabled(current >= total)
            .on_click(move |_, window, cx| on_page((current + 1).min(total), window, cx))
    };
    let row = page_buttons(current, total)
        .into_iter()
        .enumerate()
        .map(|(slot, button)| match button {
            PageButton::Page(page) => {
                let on_page = Rc::clone(&on_page);
                Button::new(("page", page))
                    .compact()
                    .label(page.to_string())
                    .when(page == current, |button| button.primary())
                    .when(page != current, |button| button.ghost())
                    .on_click(move |_, window, cx| on_page(page, window, cx))
                    .into_any_element()
            }
            PageButton::Ellipsis => Button::new(("page-gap", slot))
                .compact()
                .ghost()
                .label("...")
                .disabled(true)
                .into_any_element(),
        });
    h_flex()
        .w_full()
        .h(px(BAR_HEIGHT))
        .flex_shrink_0()
        .justify_center()
        .items_center()
        .gap_1()
        .child(previous)
        .children(row)
        .child(next)
        .child(
            div()
                .w(px(72.))
                .ml_2()
                .child(Input::new(jump).aria_label(tr("Portal.Pagination.PageJump"))),
        )
        .child(
            Button::new("page-jump")
                .outline()
                .label(tr("Portal.Pagination.PageJump"))
                .on_click(move |_, window, cx| on_jump(window, cx)),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{
        PageButton::{self, Ellipsis, Page},
        clamp_jump, jump_text_allowed, page_buttons,
    };

    fn pages(list: &[usize]) -> Vec<PageButton> {
        list.iter().map(|&page| Page(page)).collect()
    }

    #[test]
    fn pagination_bar_is_hidden_at_one_page_or_fewer() {
        assert!(page_buttons(1, 1).is_empty());
        assert!(page_buttons(1, 0).is_empty());
    }

    #[test]
    fn pagination_lists_every_page_up_to_six() {
        assert_eq!(page_buttons(1, 2), pages(&[1, 2]));
        assert_eq!(page_buttons(3, 6), pages(&[1, 2, 3, 4, 5, 6]));
    }

    #[test]
    fn pagination_scheme_near_the_start() {
        let expected = vec![
            Page(1),
            Page(2),
            Page(3),
            Page(4),
            Page(5),
            Ellipsis,
            Ellipsis,
            Page(19),
            Page(20),
        ];
        assert_eq!(page_buttons(1, 20), expected);
        assert_eq!(page_buttons(4, 20), expected);
        // Seven pages is the smallest count that uses the scheme.
        assert_eq!(
            page_buttons(1, 7),
            vec![
                Page(1),
                Page(2),
                Page(3),
                Page(4),
                Page(5),
                Ellipsis,
                Ellipsis,
                Page(6),
                Page(7)
            ]
        );
    }

    #[test]
    fn pagination_scheme_in_the_middle() {
        assert_eq!(
            page_buttons(10, 20),
            vec![
                Page(1),
                Page(2),
                Ellipsis,
                Page(9),
                Page(10),
                Page(11),
                Ellipsis,
                Page(19),
                Page(20)
            ]
        );
        assert_eq!(
            page_buttons(5, 20),
            vec![
                Page(1),
                Page(2),
                Ellipsis,
                Page(4),
                Page(5),
                Page(6),
                Ellipsis,
                Page(19),
                Page(20)
            ]
        );
        assert_eq!(
            page_buttons(16, 20),
            vec![
                Page(1),
                Page(2),
                Ellipsis,
                Page(15),
                Page(16),
                Page(17),
                Ellipsis,
                Page(19),
                Page(20)
            ]
        );
    }

    #[test]
    fn pagination_scheme_near_the_end() {
        let expected = vec![
            Page(1),
            Page(2),
            Ellipsis,
            Ellipsis,
            Page(16),
            Page(17),
            Page(18),
            Page(19),
            Page(20),
        ];
        assert_eq!(page_buttons(17, 20), expected);
        assert_eq!(page_buttons(20, 20), expected);
    }

    #[test]
    fn pagination_never_shows_more_than_nine_slots_and_always_the_first_and_last_page() {
        for total in 2..=60 {
            for current in 1..=total {
                let row = page_buttons(current, total);
                assert!(row.len() <= 9, "{current}/{total}: {row:?}");
                assert_eq!(row.first(), Some(&Page(1)), "{current}/{total}");
                assert_eq!(row.last(), Some(&Page(total)), "{current}/{total}");
                if total > 6 {
                    assert!(row.contains(&Page(current)), "{current}/{total}: {row:?}");
                }
            }
        }
    }

    #[test]
    fn pagination_jump_clamps_into_the_page_range() {
        assert_eq!(clamp_jump("", 5), None);
        assert_eq!(clamp_jump("  ", 5), None);
        assert_eq!(clamp_jump("0", 5), Some(1));
        assert_eq!(clamp_jump("99", 5), Some(5));
        assert_eq!(clamp_jump("3", 5), Some(3));
        assert_eq!(clamp_jump("x", 5), None);
        // A number too large for any integer type still lands on the last page.
        assert_eq!(clamp_jump("99999999999999999999999999999999999999999", 5), Some(5));
    }

    #[test]
    fn pagination_jump_field_takes_at_most_the_digits_of_the_last_page() {
        assert!(jump_text_allowed("", 25));
        assert!(jump_text_allowed("2", 25));
        assert!(jump_text_allowed("25", 25));
        assert!(!jump_text_allowed("250", 25));
        assert!(!jump_text_allowed("2a", 25));
        assert!(!jump_text_allowed("-1", 25));
        assert!(jump_text_allowed("9", 9));
        assert!(!jump_text_allowed("10", 9));
    }
}
