//! The pagination bar of the library page (LIB-04, UI-SPEC E5): the page buttons follow the legacy control's scheme and
//! the jump field clamps what the user types.

/// One slot of the page row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageButton {
    /// A page number button.
    Page(usize),
    /// A disabled "..." slot where pages are skipped.
    Ellipsis,
}

/// The page row for `current` of `total` pages, in the legacy control's scheme.
pub fn page_buttons(_current: usize, _total: usize) -> Vec<PageButton> {
    Vec::new()
}

/// The page a jump field text asks for, clamped to 1..=`total`; `None` for an empty or non-numeric text.
pub fn clamp_jump(_text: &str, _total: usize) -> Option<usize> {
    None
}

/// Whether the jump field may hold `text`: digits only, at most as many as the last page number has.
pub fn jump_text_allowed(_text: &str, _total: usize) -> bool {
    false
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
