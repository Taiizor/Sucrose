//! The legacy prerequisite thresholds and how a found version is compared with them (D-15, ENG-06). Pure text
//! comparison, the same on every OS: reading the registry or asking a runtime for its version is the platform's job.

/// The oldest Visual C++ Redistributable CEF accepts, as the legacy app required (`engines.md` §9).
pub const VC_REDIST_MIN: &str = "14.40.33816.0";

/// The WebView2 Runtime has to be newer than this (legacy threshold, `engines.md` §9).
pub const WEBVIEW2_FLOOR: &str = "131.0.2903.70";

/// Whether `found` is `needed` or newer.
pub fn version_at_least(found: &str, needed: &str) -> bool {
    let _ = (found, needed);
    false
}

/// Whether `found` is newer than `floor` (equal does not count).
pub fn version_newer_than(found: &str, floor: &str) -> bool {
    let _ = (found, floor);
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_least_accepts_the_threshold_itself_and_anything_newer() {
        assert!(version_at_least("14.40.33816.0", VC_REDIST_MIN));
        assert!(version_at_least("14.44.35211.0", VC_REDIST_MIN));
        assert!(version_at_least("15.0.0.0", VC_REDIST_MIN));
    }

    #[test]
    fn at_least_refuses_anything_older_component_by_component() {
        assert!(!version_at_least("14.40.33815.9", VC_REDIST_MIN));
        assert!(!version_at_least("14.39.99999.0", VC_REDIST_MIN));
        assert!(!version_at_least("13.99.99999.99", VC_REDIST_MIN));
    }

    #[test]
    fn the_registry_spelling_with_a_leading_v_and_padded_zeros_counts() {
        assert!(version_at_least("v14.44.35211.0", VC_REDIST_MIN));
        assert!(version_at_least("v14.44.35211.00", VC_REDIST_MIN));
        assert!(version_at_least("V14.40.33816.00", VC_REDIST_MIN));
    }

    #[test]
    fn missing_components_count_as_zero() {
        assert!(version_at_least("14.40.33816", VC_REDIST_MIN));
        assert!(!version_at_least("14.40", VC_REDIST_MIN));
        assert!(version_at_least("14.41", VC_REDIST_MIN));
    }

    #[test]
    fn text_that_is_not_a_version_is_never_at_least_anything() {
        assert!(!version_at_least("garbage", VC_REDIST_MIN));
        assert!(!version_at_least("", VC_REDIST_MIN));
        assert!(!version_at_least("14.x.1", VC_REDIST_MIN));
        assert!(!version_at_least("99999999999999999999999.0", VC_REDIST_MIN));
        assert!(!version_at_least("14.40.33816.0", "garbage"));
        assert!(!version_newer_than("garbage", WEBVIEW2_FLOOR));
    }

    #[test]
    fn newer_than_excludes_the_floor_itself() {
        assert!(!version_newer_than("131.0.2903.70", WEBVIEW2_FLOOR));
        assert!(version_newer_than("131.0.2903.71", WEBVIEW2_FLOOR));
        assert!(version_newer_than("158.0.4370.33", WEBVIEW2_FLOOR));
        assert!(!version_newer_than("131.0.2903.69", WEBVIEW2_FLOOR));
        assert!(!version_newer_than("130.9.9999.99", WEBVIEW2_FLOOR));
    }

    #[test]
    fn the_thresholds_are_the_legacy_ones() {
        assert_eq!(VC_REDIST_MIN, "14.40.33816.0");
        assert_eq!(WEBVIEW2_FLOOR, "131.0.2903.70");
    }
}
