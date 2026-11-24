//! The status bar (UI-SPEC E10, D-19): the build, the memory of all Sucrose processes as the service reports it, a quote
//! that copies on click and the copyright. Below 1000 px the copyright hides first, then the quote.

use sucrose_ipc::portal::MemoryReport;

/// Narrower than this (logical px) the copyright is hidden.
pub const COPYRIGHT_MIN_WIDTH: f32 = 1000.0;
/// Narrower than this (logical px) the quote is hidden too.
pub const QUOTE_MIN_WIDTH: f32 = 900.0;
/// How many quotes the locale files hold (Portal.Quoting0 .. Portal.Quoting85).
pub const QUOTE_COUNT: usize = 86;

/// Which optional groups of the bar fit the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleGroups {
    /// The quote.
    pub quote: bool,
    /// The copyright.
    pub copyright: bool,
}

/// What a click on the quote ended in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyState {
    /// Nothing was copied lately.
    Idle,
    /// The text is on the clipboard.
    Copied,
    /// The clipboard did not take the text.
    Failed,
}

/// The total of all Sucrose processes in bytes.
pub fn memory_total(report: &MemoryReport) -> u64 {
    let _ = report;
    0
}

/// The memory text of the bar: "—" without a report, else the total in MB.
pub fn memory_text(report: Option<&MemoryReport>) -> String {
    let _ = report;
    String::new()
}

/// The tooltip lines of the memory text.
pub fn memory_tooltip(report: Option<&MemoryReport>) -> Option<String> {
    let _ = report;
    None
}

/// Which groups fit a window of this width.
pub fn visible_groups(width: f32) -> VisibleGroups {
    let _ = width;
    VisibleGroups {
        quote: true,
        copyright: true,
    }
}

/// The architecture as the legacy bar names it.
pub fn arch_text(arch: &str) -> &str {
    let _ = arch;
    ""
}

/// The quote number a seed picks.
pub fn quote_index(seed: u64) -> usize {
    let _ = seed;
    0
}

/// Whether the copy worked: the clipboard must hold the text afterwards.
pub fn copy_outcome(expected: &str, read_back: Option<&str>) -> CopyState {
    let _ = (expected, read_back);
    CopyState::Idle
}

#[cfg(test)]
mod tests {
    use sucrose_ipc::portal::MemoryReport;

    use super::{
        CopyState, QUOTE_COUNT, arch_text, copy_outcome, memory_text, memory_tooltip, memory_total, quote_index,
        visible_groups,
    };

    const MIB: u64 = 1024 * 1024;

    fn report() -> MemoryReport {
        MemoryReport {
            service: 100 * MIB,
            renderers: vec![30 * MIB, 10 * MIB],
            portal: 10 * MIB,
        }
    }

    #[test]
    fn the_total_adds_every_process() {
        assert_eq!(memory_total(&report()), 150 * MIB);
        let empty = MemoryReport {
            service: 0,
            renderers: Vec::new(),
            portal: 0,
        };
        assert_eq!(memory_total(&empty), 0);
        let huge = MemoryReport {
            service: u64::MAX,
            renderers: vec![u64::MAX],
            portal: 1,
        };
        assert_eq!(memory_total(&huge), u64::MAX, "the sum saturates instead of wrapping");
    }

    #[test]
    fn no_report_shows_a_dash() {
        assert_eq!(memory_text(None), "—");
        assert_eq!(memory_tooltip(None), None);
    }

    #[test]
    fn the_total_shows_in_whole_megabytes() {
        assert_eq!(memory_text(Some(&report())), "150 MB");
        let small = MemoryReport {
            service: MIB + MIB / 2,
            renderers: Vec::new(),
            portal: 0,
        };
        assert_eq!(memory_text(Some(&small)), "2 MB", "a half rounds up");
        let none = MemoryReport {
            service: 0,
            renderers: Vec::new(),
            portal: 0,
        };
        assert_eq!(memory_text(Some(&none)), "0 MB");
    }

    #[test]
    fn the_tooltip_lists_every_process_and_the_total() {
        let tip = memory_tooltip(Some(&report())).expect("a report has a tooltip");
        let lines: Vec<&str> = tip.lines().collect();
        assert_eq!(
            lines,
            [
                "Service: 100 MB",
                "Renderer 1: 30 MB",
                "Renderer 2: 10 MB",
                "Portal: 10 MB",
                "Total: 150 MB"
            ]
        );
        let alone = MemoryReport {
            service: 5 * MIB,
            renderers: Vec::new(),
            portal: 7 * MIB,
        };
        let tip = memory_tooltip(Some(&alone)).expect("a report has a tooltip");
        assert_eq!(tip.lines().count(), 3, "no renderer, no renderer line");
    }

    #[test]
    fn narrow_windows_hide_the_copyright_first_and_then_the_quote() {
        let wide = visible_groups(1200.0);
        assert!(wide.quote && wide.copyright);
        let edge = visible_groups(1000.0);
        assert!(edge.quote && edge.copyright, "1000 px still fits both");
        let narrow = visible_groups(999.0);
        assert!(narrow.quote && !narrow.copyright);
        let narrower = visible_groups(899.0);
        assert!(!narrower.quote && !narrower.copyright);
    }

    #[test]
    fn the_architecture_has_the_legacy_names() {
        assert_eq!(arch_text("x86_64"), "x64");
        assert_eq!(arch_text("aarch64"), "arm64");
        assert_eq!(arch_text("x86"), "x86");
        assert_eq!(
            arch_text("riscv64"),
            "riscv64",
            "an unknown architecture shows as it is"
        );
    }

    #[test]
    fn a_seed_always_picks_an_existing_quote() {
        for seed in [0, 1, 85, 86, 87, u64::MAX] {
            assert!(quote_index(seed) < QUOTE_COUNT, "{seed}");
        }
        assert_eq!(quote_index(0), 0);
        assert_eq!(quote_index(86), 0);
        assert_eq!(quote_index(85), 85);
    }

    #[test]
    fn a_copy_counts_only_when_the_clipboard_holds_the_text() {
        assert_eq!(copy_outcome("Love", Some("Love")), CopyState::Copied);
        assert_eq!(copy_outcome("Love", Some("other")), CopyState::Failed);
        assert_eq!(copy_outcome("Love", None), CopyState::Failed);
    }
}
