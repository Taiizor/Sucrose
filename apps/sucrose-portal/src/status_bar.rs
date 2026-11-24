//! The status bar (UI-SPEC E10, D-19): the build, the memory of all Sucrose processes as the service reports it, a quote
//! that copies on click and the copyright. Below 1000 px the copyright hides first, then the quote.
//!
//! Groups follow each other in flex order, never by left and right geometry, so the bar can mirror for Arabic and Hebrew
//! (UI-SPEC E4 RTL row; the mirrored screenshot is due with UI-06).

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gpui_kit::assets::IconName;
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, h_flex, tooltip::Tooltip};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use sucrose_ipc::portal::MemoryReport;

use crate::i18n::tr;

/// Height of the bar (UI-SPEC spacing).
const STATUS_BAR_HEIGHT: f32 = 32.0;
/// Narrower than this (logical px) the copyright is hidden.
pub const COPYRIGHT_MIN_WIDTH: f32 = 1000.0;
/// Narrower than this (logical px) the quote is hidden too. The window never opens narrower than 900 px, so this
/// only matters for a window manager that ignores the minimum size.
pub const QUOTE_MIN_WIDTH: f32 = 900.0;
/// How many quotes the locale files hold (Portal.Quoting0 .. Portal.Quoting85).
pub const QUOTE_COUNT: usize = 86;
/// How long "Copied" (or the failure) stays in place of the quote.
const COPY_FEEDBACK: Duration = Duration::from_secs(2);
/// Bytes in the unit the legacy bar calls MB.
const MIB: u64 = 1024 * 1024;

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
    report
        .renderers
        .iter()
        .fold(report.service.saturating_add(report.portal), |sum, bytes| {
            sum.saturating_add(*bytes)
        })
}

/// Whole MB of a byte count; a half rounds up.
fn megabytes(bytes: u64) -> u64 {
    bytes / MIB + u64::from(bytes % MIB >= MIB / 2)
}

/// "{n} MB" for a byte count (the unit is the legacy text, not localized).
fn mb_text(bytes: u64) -> String {
    format!("{} MB", megabytes(bytes))
}

/// The memory text of the bar: "—" without a report, else the total in MB.
pub fn memory_text(report: Option<&MemoryReport>) -> String {
    report.map_or_else(|| "—".to_owned(), |report| mb_text(memory_total(report)))
}

/// The tooltip lines of the memory text: the service, each renderer, the Portal and the total. None without a report.
pub fn memory_tooltip(report: Option<&MemoryReport>) -> Option<String> {
    let report = report?;
    let mut lines = vec![format!(
        "{}: {}",
        tr("Portal.StatusBar.MemoryService"),
        mb_text(report.service)
    )];
    let renderer = tr("Portal.StatusBar.MemoryRenderer");
    for (index, bytes) in report.renderers.iter().enumerate() {
        lines.push(format!("{renderer} {}: {}", index + 1, mb_text(*bytes)));
    }
    lines.push(format!(
        "{}: {}",
        tr("Portal.StatusBar.MemoryPortal"),
        mb_text(report.portal)
    ));
    lines.push(format!(
        "{}: {}",
        tr("Portal.StatusBar.MemoryTotal"),
        mb_text(memory_total(report))
    ));
    Some(lines.join("\n"))
}

/// Which groups fit a window of this width.
pub fn visible_groups(width: f32) -> VisibleGroups {
    VisibleGroups {
        quote: width >= QUOTE_MIN_WIDTH,
        copyright: width >= COPYRIGHT_MIN_WIDTH,
    }
}

/// The architecture as the legacy bar names it; one it has no name for shows as it is.
pub fn arch_text(arch: &str) -> &str {
    match arch {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    }
}

/// The quote number a seed picks.
pub fn quote_index(seed: u64) -> usize {
    usize::try_from(seed % QUOTE_COUNT as u64).unwrap_or(0)
}

/// Whether the copy worked: the clipboard must hold the text afterwards (GPUI's write returns nothing, so the read back
/// is the only evidence).
pub fn copy_outcome(expected: &str, read_back: Option<&str>) -> CopyState {
    if read_back == Some(expected) {
        CopyState::Copied
    } else {
        CopyState::Failed
    }
}

/// The status bar entity: the memory report, the quote picked at start and the copy feedback.
pub struct StatusBar {
    memory: Option<MemoryReport>,
    quote: SharedString,
    copy: CopyState,
    /// The pending return from the copy feedback to the quote; replacing it restarts the 2 seconds.
    reset: Option<Task<()>>,
}

impl StatusBar {
    /// A bar with a random quote and no memory report yet.
    pub fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| u64::try_from(elapsed.as_nanos()).unwrap_or(0));
        Self {
            memory: None,
            quote: tr(&format!("Portal.Quoting{}", quote_index(seed))),
            copy: CopyState::Idle,
            reset: None,
        }
    }

    /// The latest memory report, or None while the service is unreachable (the text shows "—").
    pub fn set_memory(&mut self, report: Option<MemoryReport>, cx: &mut Context<Self>) {
        if self.memory != report {
            self.memory = report;
            cx.notify();
        }
    }

    /// Puts the quote on the clipboard and shows the result for two seconds.
    fn copy_quote(&mut self, cx: &mut Context<Self>) {
        let text = self.quote.to_string();
        cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
        let read_back = cx.read_from_clipboard().and_then(|item| item.text());
        self.copy = copy_outcome(&text, read_back.as_deref());
        if self.copy == CopyState::Failed {
            tracing::warn!("the clipboard did not take the quote");
        }
        let timer = cx.background_executor().timer(COPY_FEEDBACK);
        self.reset = Some(cx.spawn(async move |this, cx| {
            timer.await;
            this.update(cx, |this, cx| {
                this.copy = CopyState::Idle;
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }
}

impl Render for StatusBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let groups = visible_groups(f32::from(window.viewport_size().width));
        let memory_tip = memory_tooltip(self.memory.as_ref());
        let quote_tip = self.quote.clone();
        let quote_shown = match self.copy {
            CopyState::Idle => self.quote.clone(),
            CopyState::Copied => tr("Portal.StatusBar.Copied"),
            CopyState::Failed => tr("Portal.StatusBar.CopyFailed"),
        };
        h_flex()
            .w_full()
            .h(px(STATUS_BAR_HEIGHT))
            .flex_shrink_0()
            .px_4()
            .gap_4()
            .items_center()
            .text_xs()
            .text_color(muted)
            .bg(cx.theme().secondary.opacity(0.5))
            .child(div().flex_none().child(Icon::new(IconName::LayoutGrid).small()))
            .child(div().flex_none().whitespace_nowrap().child(tr("Portal.Release")))
            .child(div().flex_none().whitespace_nowrap().child("Rust"))
            .child(
                div()
                    .flex_none()
                    .whitespace_nowrap()
                    .child(arch_text(std::env::consts::ARCH).to_owned()),
            )
            .child(
                div()
                    .flex_none()
                    .whitespace_nowrap()
                    .child(format!("v{}", crate::version_text())),
            )
            .child(
                div()
                    .id("status-memory")
                    .flex_none()
                    .whitespace_nowrap()
                    .when_some(memory_tip, |item, tip| {
                        item.tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
                    })
                    .child(memory_text(self.memory.as_ref())),
            )
            .child(if groups.quote {
                div()
                    .id("status-quote")
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .cursor_pointer()
                    .tooltip(move |window, cx| Tooltip::new(quote_tip.clone()).build(window, cx))
                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.copy_quote(cx)))
                    .child(div().truncate().child(quote_shown))
                    .into_any_element()
            } else {
                // The spacer keeps the copyright at the end of the row.
                div().flex_1().into_any_element()
            })
            .when(groups.copyright, |bar| {
                bar.child(div().flex_none().whitespace_nowrap().child(tr("Portal.Copyright")))
            })
    }
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
