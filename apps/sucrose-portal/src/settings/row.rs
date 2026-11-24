//! The expander row of the settings pages and the rules for committing, clamping and reverting a value.

/// The number a numeric field commits: its text clamped to `min..=max`; an empty or non-numeric text keeps `previous`.
pub fn commit_number(_text: &str, _min: u8, _max: u8, previous: u8) -> u8 {
    previous
}

#[cfg(test)]
mod tests {
    use super::commit_number;

    #[test]
    fn settings_number_empty_text_keeps_the_previous_value() {
        assert_eq!(commit_number("", 1, 100, 30), 30);
        assert_eq!(commit_number("   ", 1, 100, 30), 30);
    }

    #[test]
    fn settings_number_below_the_range_is_raised_to_the_minimum() {
        assert_eq!(commit_number("0", 1, 100, 30), 1);
        assert_eq!(commit_number("3", 5, 25, 10), 5);
    }

    #[test]
    fn settings_number_above_the_range_is_cut_to_the_maximum() {
        assert_eq!(commit_number("250", 1, 100, 30), 100);
        assert_eq!(commit_number("99999999999999999999", 1, 100, 30), 100);
    }

    #[test]
    fn settings_number_inside_the_range_is_kept() {
        assert_eq!(commit_number("42", 1, 100, 30), 42);
        assert_eq!(commit_number(" 7 ", 0, 100, 30), 7);
        assert_eq!(commit_number("0", 0, 100, 30), 0);
    }

    #[test]
    fn settings_number_that_is_not_a_number_keeps_the_previous_value() {
        assert_eq!(commit_number("4a", 1, 100, 30), 30);
        assert_eq!(commit_number("-5", 1, 100, 30), 30);
        assert_eq!(commit_number("1.5", 1, 100, 30), 30);
    }
}
