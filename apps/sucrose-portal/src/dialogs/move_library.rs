//! The Move Library confirmation (LIB-09, D-07): shown only when the Move switch is on. Move Library is the one destructive
//! action of the Portal (UI-SPEC Color: destructive, reserved use 1), Keep Current Location closes the dialog and changes
//! nothing. When the classic Sucrose app is installed the dialog adds that it keeps the old location and will see an empty
//! library; Sucrose never writes the classic app's own settings.

use std::rc::Rc;

use gpui_kit::component::{
    WindowExt as _,
    button::{Button, ButtonVariants as _},
    dialog::DialogFooter,
    v_flex,
};
use gpui_kit::*;
use sucrose_core::paths::legacy_setting_dir;

use super::{DIALOG_WIDTH, body};
use crate::i18n::{tr, tr_with};

/// What runs when Move Library is pressed.
type ConfirmHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// Whether the classic Sucrose app is installed: its `Setting` folder exists. Only the existence of the folder is looked
/// at, no file in it is read (D-07). It touches the disk, so call it from the background executor.
pub fn classic_app_present() -> bool {
    legacy_setting_dir().is_some_and(|dir| dir.is_dir())
}

/// The paragraphs of the dialog: the question with the destination, and the classic-app sentence when it applies.
pub fn paragraphs(destination: &str, classic_app: bool) -> Vec<String> {
    let mut paragraphs = vec![
        tr_with(
            "Portal.GeneralSettingPage.PrivateLibrary.ConfirmBody",
            &[("path", destination)],
        )
        .to_string(),
    ];
    if classic_app {
        paragraphs.push(tr("Portal.GeneralSettingPage.PrivateLibrary.ConfirmLegacy").to_string());
    }
    paragraphs
}

/// Opens the confirmation for `destination`. `on_confirm` runs when Move Library is pressed, after the dialog closed.
pub fn open(
    destination: String,
    classic_app: bool,
    on_confirm: impl Fn(&mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut App,
) {
    let on_confirm: ConfirmHandler = Rc::new(on_confirm);
    window.open_dialog(cx, move |dialog, window, _cx| {
        let rows = paragraphs(&destination, classic_app)
            .into_iter()
            .map(|text| div().text_sm().child(text).into_any_element());
        let on_confirm = on_confirm.clone();
        dialog
            .title(tr("Portal.GeneralSettingPage.PrivateLibrary.ConfirmTitle"))
            .w(px(DIALOG_WIDTH))
            .child(body(window, [v_flex().gap_3().children(rows).into_any_element()]))
            .footer(
                DialogFooter::new()
                    .child(
                        Button::new("move-library-keep")
                            .label(tr("Portal.GeneralSettingPage.PrivateLibrary.KeepLocation"))
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("move-library-confirm")
                            .danger()
                            .label(tr("Portal.GeneralSettingPage.PrivateLibrary.Confirm"))
                            .on_click(move |_, window, cx| {
                                window.close_dialog(cx);
                                on_confirm(window, cx);
                            }),
                    ),
            )
    });
}

#[cfg(test)]
mod tests {
    use super::paragraphs;

    #[test]
    fn the_question_names_the_destination() {
        let text = paragraphs("D:\\Library", false);

        assert_eq!(
            text,
            [
                "Move your themes to D:\\Library? They are copied there, the old folder is removed and the wallpaper restarts."
            ]
        );
    }

    #[test]
    fn a_classic_install_adds_that_it_will_see_an_empty_library() {
        let text = paragraphs("D:\\Library", true);

        assert_eq!(text.len(), 2);
        assert_eq!(
            text[1],
            "The classic Sucrose app keeps using the old location and will see an empty library."
        );
    }
}
