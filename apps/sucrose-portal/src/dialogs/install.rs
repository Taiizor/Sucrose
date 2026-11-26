//! The install dialog (UI-SPEC "Install dialog", E2, E3, D-15): the user confirms, the service downloads the Microsoft
//! installer, checks its signature, runs it and looks again, and the dialog follows every stage it reports. The
//! confirmation comes first: nothing is downloaded before Download and Install is pressed. Escape and a click outside
//! close the dialog only in the first state and in the failure states; while a download runs the only way out is Stop
//! Download, and while the signature is checked or the installer runs there is none.

use std::{cell::RefCell, rc::Rc};

use gpui_kit::component::{
    ActiveTheme as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    dialog::DialogFooter,
    progress::Progress,
    v_flex,
};
use gpui_kit::*;
use sucrose_ipc::portal::{InstallFailureView, InstallStageView, PortalRequest, PrerequisiteView};

use super::{DIALOG_WIDTH, body};
use crate::{
    i18n::{tr, tr_with},
    ipc::ServiceLink,
};

/// Where the dialog is, from what the user did and what the service reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// The confirmation: nothing has been asked of the service yet.
    Confirm,
    /// The installer is being downloaded.
    Downloading {
        /// Bytes received so far.
        received: u64,
        /// Bytes expected, when known.
        total: Option<u64>,
    },
    /// The signature is being checked.
    Verifying,
    /// The installer runs.
    Installing,
    /// The part is being looked for again.
    Rechecking,
    /// The install stopped.
    Failed(InstallFailureView),
}

impl Stage {
    /// The stage a report of the service puts the dialog in; `None` for the two ends that close it (installed, stopped).
    pub fn from_view(view: InstallStageView) -> Option<Self> {
        match view {
            InstallStageView::Downloading { received, total } => Some(Self::Downloading { received, total }),
            InstallStageView::Verifying => Some(Self::Verifying),
            InstallStageView::Installing => Some(Self::Installing),
            InstallStageView::Rechecking => Some(Self::Rechecking),
            InstallStageView::Failed(failure) => Some(Self::Failed(failure)),
            InstallStageView::Done | InstallStageView::Cancelled => None,
        }
    }
}

/// A button of the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Download and Install: asks the service to start.
    Install,
    /// Don't Install: closes the dialog.
    DontInstall,
    /// Stop Download: asks the service to stop and closes the dialog.
    StopDownload,
    /// Try Again after a failure: asks the service to start again.
    TryAgain,
    /// Close after a failure.
    Close,
}

/// What the status area shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    /// Nothing (the confirmation).
    None,
    /// A progress bar with the line under it; `percent` is `None` while the size is unknown (an indeterminate bar).
    Bar {
        /// 0 to 100, when the total is known.
        percent: Option<f32>,
        /// The key of the line.
        line: &'static str,
    },
    /// One Label line.
    Line(&'static str),
    /// The failure copy, in place of the body.
    Error(&'static str),
}

/// What the dialog shows in one stage: the body key, the status, the buttons in the order they are drawn (the primary
/// one last) and whether Escape and a click outside close it.
#[derive(Debug, Clone, PartialEq)]
pub struct DialogState {
    /// The key of the explanation shown above the status; `None` once a failure replaces it.
    pub body: Option<&'static str>,
    /// The status area.
    pub status: Status,
    /// The buttons, left to right.
    pub actions: &'static [Action],
    /// Whether Escape and a click outside close the dialog.
    pub escape_closes: bool,
}

/// The key of the failure copy.
fn failure_key(failure: InstallFailureView) -> &'static str {
    match failure {
        InstallFailureView::Download => "Portal.EngineInstall.DownloadFailed",
        InstallFailureView::Signature => "Portal.EngineInstall.SignatureFailed",
        InstallFailureView::Installer => "Portal.EngineInstall.InstallFailed",
    }
}

/// How far a download is, in percent; `None` while the total is unknown or zero.
#[allow(
    clippy::cast_precision_loss,
    reason = "a progress bar needs no more precision than f32 gives"
)]
fn percent(received: u64, total: Option<u64>) -> Option<f32> {
    total
        .filter(|total| *total > 0)
        .map(|total| (received.min(total) as f64 / total as f64 * 100.0) as f32)
}

/// What the dialog shows in `stage` (UI-SPEC E2: loading and error).
pub fn dialog_state(stage: Stage) -> DialogState {
    const BODY: Option<&str> = Some("Portal.EngineInstall.Body");
    match stage {
        Stage::Confirm => DialogState {
            body: BODY,
            status: Status::None,
            actions: &[Action::DontInstall, Action::Install],
            escape_closes: true,
        },
        Stage::Downloading { received, total } => DialogState {
            body: BODY,
            status: Status::Bar {
                percent: percent(received, total),
                line: "Portal.EngineInstall.Downloading",
            },
            actions: &[Action::StopDownload],
            escape_closes: false,
        },
        Stage::Verifying => busy("Portal.EngineInstall.Verifying"),
        Stage::Installing => busy("Portal.EngineInstall.Installing"),
        Stage::Rechecking => busy("Portal.EngineInstall.Rechecking"),
        Stage::Failed(failure) => DialogState {
            body: None,
            status: Status::Error(failure_key(failure)),
            actions: &[Action::Close, Action::TryAgain],
            escape_closes: true,
        },
    }
}

/// A stage with one status line and no buttons.
fn busy(line: &'static str) -> DialogState {
    DialogState {
        body: Some("Portal.EngineInstall.Body"),
        status: Status::Line(line),
        actions: &[],
        escape_closes: false,
    }
}

/// The key of the text that names a prerequisite in the title.
pub fn prerequisite_key(prerequisite: PrerequisiteView) -> &'static str {
    match prerequisite {
        PrerequisiteView::VcRedist => "Portal.EngineInstall.Prerequisite.VcRedist",
        PrerequisiteView::WebView2Runtime => "Portal.EngineInstall.Prerequisite.WebView2Runtime",
    }
}

/// What the open dialog and the Portal's event handler share.
struct Shared {
    prerequisite: PrerequisiteView,
    stage: Stage,
}

/// The dialog that is open, if any; the Portal's event handler reads it to route the service's reports.
#[derive(Default)]
struct Open(Option<Rc<RefCell<Shared>>>);

impl Global for Open {}

/// Opens the install dialog for `prerequisite` on the confirmation. Pressing Download and Install is what starts the
/// service; closing the dialog any other way asks for nothing.
pub fn open_install_dialog(prerequisite: PrerequisiteView, link: ServiceLink, window: &mut Window, cx: &mut App) {
    let shared = Rc::new(RefCell::new(Shared {
        prerequisite,
        stage: Stage::Confirm,
    }));
    cx.set_global(Open(Some(Rc::clone(&shared))));
    window.open_dialog(cx, move |dialog, window, cx| {
        let (prerequisite, stage) = {
            let shared = shared.borrow();
            (shared.prerequisite, shared.stage)
        };
        let state = dialog_state(stage);
        let muted = cx.theme().muted_foreground;
        let name = tr(prerequisite_key(prerequisite));
        let mut rows: Vec<AnyElement> = Vec::new();
        if let Some(key) = state.body {
            rows.push(div().text_sm().child(tr(key)).into_any_element());
        }
        match state.status {
            Status::None => {}
            Status::Bar { percent, line } => rows.push(
                v_flex()
                    .gap_2()
                    .child(
                        Progress::new("install-progress")
                            .h(px(8.))
                            .loading(percent.is_none())
                            .value(percent.unwrap_or(0.))
                            .accessibility_label(tr(line)),
                    )
                    .child(div().text_xs().text_color(muted).child(tr(line)))
                    .into_any_element(),
            ),
            Status::Line(line) => rows.push(div().text_xs().text_color(muted).child(tr(line)).into_any_element()),
            Status::Error(key) => rows.push(div().text_sm().child(tr(key)).into_any_element()),
        }
        let mut footer = DialogFooter::new();
        for action in state.actions {
            footer = footer.child(action_button(*action, &shared, &link));
        }
        let mut dialog = dialog
            .title(tr_with("Portal.EngineInstall.Title", &[("prerequisite", &name)]))
            .w(px(DIALOG_WIDTH))
            .keyboard(state.escape_closes)
            .overlay_closable(state.escape_closes)
            .close_button(state.escape_closes)
            .child(body(window, [v_flex().gap_3().children(rows).into_any_element()]));
        if !state.actions.is_empty() {
            dialog = dialog.footer(footer);
        }
        dialog
    });
}

/// The button of `action`.
fn action_button(action: Action, shared: &Rc<RefCell<Shared>>, link: &ServiceLink) -> Button {
    let (id, key) = match action {
        Action::Install => ("install-confirm", "Portal.EngineInstall.Confirm"),
        Action::DontInstall => ("install-decline", "Portal.EngineInstall.Decline"),
        Action::StopDownload => ("install-stop", "Portal.EngineInstall.StopDownload"),
        Action::TryAgain => ("install-retry", "Portal.EngineInstall.TryAgain"),
        Action::Close => ("install-close", "Portal.EngineInstall.Close"),
    };
    let button = Button::new(id).label(tr(key));
    let shared = Rc::clone(shared);
    let link = link.clone();
    match action {
        Action::Install | Action::TryAgain => button.primary().on_click(move |_, window, _| {
            let prerequisite = {
                let mut shared = shared.borrow_mut();
                shared.stage = Stage::Downloading {
                    received: 0,
                    total: None,
                };
                shared.prerequisite
            };
            link.send(PortalRequest::InstallPrerequisite { prerequisite });
            window.refresh();
        }),
        Action::StopDownload => button.outline().on_click(move |_, window, cx| {
            link.send(PortalRequest::CancelInstall);
            cx.set_global(Open(None));
            window.close_dialog(cx);
        }),
        Action::DontInstall | Action::Close => button.outline().on_click(move |_, window, cx| {
            cx.set_global(Open(None));
            window.close_dialog(cx);
        }),
    }
}

/// The service reported a stage of the install: the open install dialog follows it, and closes on success or when the
/// download was stopped. With no install dialog open the report is ignored (the user already closed it).
pub fn on_progress(view: InstallStageView, window: &mut Window, cx: &mut App) {
    let Some(shared) = cx.try_global::<Open>().and_then(|open| open.0.clone()) else {
        return;
    };
    match Stage::from_view(view) {
        Some(stage) => {
            shared.borrow_mut().stage = stage;
            window.refresh();
        }
        None => {
            cx.set_global(Open(None));
            window.close_dialog(cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use sucrose_ipc::portal::{InstallFailureView, InstallStageView};

    use super::{Action, DialogState, Stage, Status, dialog_state, percent, prerequisite_key};
    use crate::i18n::tr;

    fn state(stage: Stage) -> DialogState {
        dialog_state(stage)
    }

    #[test]
    fn install_the_confirmation_offers_download_and_install_and_dont_install() {
        let confirm = state(Stage::Confirm);

        assert_eq!(confirm.actions, [Action::DontInstall, Action::Install]);
        assert_eq!(confirm.status, Status::None);
        assert!(confirm.escape_closes);
        assert!(confirm.body.is_some());
    }

    #[test]
    fn install_a_download_shows_the_bar_and_only_stop_download() {
        let downloading = state(Stage::Downloading {
            received: 5,
            total: Some(20),
        });

        assert_eq!(downloading.actions, [Action::StopDownload]);
        assert_eq!(
            downloading.status,
            Status::Bar {
                percent: Some(25.0),
                line: "Portal.EngineInstall.Downloading"
            }
        );
        assert!(!downloading.escape_closes);
    }

    #[test]
    fn install_an_unknown_size_gives_an_indeterminate_bar() {
        assert_eq!(percent(5, None), None);
        assert_eq!(percent(5, Some(0)), None);
        assert_eq!(percent(30, Some(20)), Some(100.0));
    }

    #[test]
    fn install_verifying_installing_and_checking_again_have_no_buttons_and_ignore_escape() {
        for (stage, line) in [
            (Stage::Verifying, "Portal.EngineInstall.Verifying"),
            (Stage::Installing, "Portal.EngineInstall.Installing"),
            (Stage::Rechecking, "Portal.EngineInstall.Rechecking"),
        ] {
            let busy = state(stage);

            assert!(busy.actions.is_empty(), "{stage:?}");
            assert!(!busy.escape_closes, "{stage:?}");
            assert_eq!(busy.status, Status::Line(line));
        }
    }

    #[test]
    fn install_each_failure_has_its_own_copy_with_try_again_and_close() {
        for (failure, key, text) in [
            (
                InstallFailureView::Download,
                "Portal.EngineInstall.DownloadFailed",
                "The installer couldn't be downloaded. Check your connection and try again.",
            ),
            (
                InstallFailureView::Signature,
                "Portal.EngineInstall.SignatureFailed",
                "The installer isn't signed by Microsoft, so Sucrose didn't run it. Nothing was installed.",
            ),
            (
                InstallFailureView::Installer,
                "Portal.EngineInstall.InstallFailed",
                "The installer didn't finish. Nothing was changed. Try again or install it from Microsoft's website.",
            ),
        ] {
            let failed = state(Stage::Failed(failure));

            assert_eq!(failed.status, Status::Error(key));
            assert_eq!(tr(key).as_ref(), text);
            assert_eq!(failed.actions, [Action::Close, Action::TryAgain]);
            assert!(failed.escape_closes);
            assert!(failed.body.is_none());
        }
    }

    #[test]
    fn install_escape_closes_only_the_first_state_and_the_failures() {
        let closes: Vec<bool> = [
            Stage::Confirm,
            Stage::Downloading {
                received: 0,
                total: None,
            },
            Stage::Verifying,
            Stage::Installing,
            Stage::Rechecking,
            Stage::Failed(InstallFailureView::Download),
        ]
        .into_iter()
        .map(|stage| state(stage).escape_closes)
        .collect();

        assert_eq!(closes, [true, false, false, false, false, true]);
    }

    #[test]
    fn install_the_service_reports_move_the_stage_and_the_two_ends_close_the_dialog() {
        assert_eq!(
            Stage::from_view(InstallStageView::Downloading {
                received: 1,
                total: Some(2)
            }),
            Some(Stage::Downloading {
                received: 1,
                total: Some(2)
            })
        );
        assert_eq!(Stage::from_view(InstallStageView::Verifying), Some(Stage::Verifying));
        assert_eq!(
            Stage::from_view(InstallStageView::Failed(InstallFailureView::Signature)),
            Some(Stage::Failed(InstallFailureView::Signature))
        );
        assert_eq!(Stage::from_view(InstallStageView::Done), None);
        assert_eq!(Stage::from_view(InstallStageView::Cancelled), None);
    }

    #[test]
    fn install_the_title_names_the_prerequisite() {
        assert_eq!(
            tr(prerequisite_key(sucrose_ipc::portal::PrerequisiteView::VcRedist)).as_ref(),
            "Microsoft Visual C++ Redistributable"
        );
        assert_eq!(
            crate::i18n::tr_with(
                "Portal.EngineInstall.Title",
                &[("prerequisite", "Microsoft Visual C++ Redistributable")]
            )
            .as_ref(),
            "Install Microsoft Visual C++ Redistributable"
        );
    }
}
