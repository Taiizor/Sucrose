//! The connection banner (UI-SPEC E6): between the header row and the content, full width, 8 x 16 padding. Its text wraps
//! onto more lines instead of being cut.

use gpui_kit::component::{alert::Alert, button::Button, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::{i18n::tr, ipc::ConnectionState};

/// The banner for `state`, or nothing while the Portal is connected. `on_retry` runs when the user presses Try Again,
/// which only the failed state offers.
pub fn connection_banner(
    state: ConnectionState,
    on_retry: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Option<AnyElement> {
    let (alert, retry) = match state {
        ConnectionState::Connected => return None,
        ConnectionState::Connecting => (
            Alert::info("connection-banner", tr("Portal.Connection.Connecting")),
            false,
        ),
        ConnectionState::Lost { .. } => (Alert::info("connection-banner", tr("Portal.Connection.Lost")), false),
        // Only the failure uses the error look: the destructive color is for what the user has to act on.
        ConnectionState::Failed => (Alert::error("connection-banner", tr("Portal.Connection.Failed")), true),
    };
    Some(
        h_flex()
            .w_full()
            .px_4()
            .py_2()
            .gap_3()
            .items_center()
            .child(alert.flex_1())
            .when(retry, |row| {
                row.child(
                    Button::new("connection-retry")
                        .label(tr("Portal.Connection.TryAgain"))
                        .on_click(on_retry),
                )
            })
            .into_any_element(),
    )
}
