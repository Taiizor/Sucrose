//! The About dialog (UI-05): the author's blurb, the product name, the version and the plain links. It has no
//! "Last Checked" line, no partners and no update status until the updater exists (Phase 5); no donate entry, GitHub
//! sign-in or token field (D-020, D-023).

use gpui_kit::assets::IconName;
use gpui_kit::component::{ActiveTheme as _, StyledExt as _, WindowExt as _, v_flex};
use gpui_kit::*;

use super::{ABOUT_WIDTH, body, close_footer, link_row, urls};
use crate::i18n::tr;

/// One link row: its element id, icon, the key of its title and the target.
struct AboutLink {
    id: &'static str,
    icon: IconName,
    title: &'static str,
    url: String,
}

/// The five links, in the legacy order; the release notes belong to this version.
fn links(version: &str) -> [AboutLink; 5] {
    [
        AboutLink {
            id: "about-release-notes",
            icon: IconName::NotebookText,
            title: "Portal.OtherAbout.Update.Notes",
            url: release_notes_url(version),
        },
        AboutLink {
            id: "about-contributors",
            icon: IconName::Users,
            title: "Portal.OtherAbout.Contributors",
            url: urls::CONTRIBUTORS.to_owned(),
        },
        AboutLink {
            id: "about-terms",
            icon: IconName::FileText,
            title: "Portal.OtherAbout.TermsOfService",
            url: urls::TERMS_OF_SERVICE.to_owned(),
        },
        AboutLink {
            id: "about-privacy",
            icon: IconName::Shield,
            title: "Portal.OtherAbout.PrivacyPolicy",
            url: urls::PRIVACY_POLICY.to_owned(),
        },
        AboutLink {
            id: "about-conduct",
            icon: IconName::ClipboardList,
            title: "Portal.OtherAbout.CodeOfConduct",
            url: urls::CODE_OF_CONDUCT.to_owned(),
        },
    ]
}

/// The page of the release notes of a version (the tag is `v{version}`, as in the legacy dialog).
pub fn release_notes_url(version: &str) -> String {
    format!("{}/v{version}", urls::RELEASE_TAG)
}

/// "Version: v{version}" in the current language; the legacy text carries a "{0}" placeholder.
pub fn version_line(version: &str) -> String {
    tr("Portal.OtherAbout.Update").replace("{0}", version)
}

/// Opens the About dialog.
pub fn open(window: &mut Window, cx: &mut App) {
    window.open_dialog(cx, |dialog, window, cx| {
        let version = crate::version_text();
        let muted = cx.theme().muted_foreground;
        let intro = v_flex()
            .gap_2()
            .pb_2()
            .child(div().text_color(muted).text_sm().child(tr("Portal.OtherAbout.About")))
            .child(div().text_2xl().font_semibold().child(crate::window_title()))
            .child(div().text_sm().child(version_line(&version)))
            .into_any_element();
        let rows = std::iter::once(intro).chain(
            links(&version)
                .into_iter()
                .map(|link| link_row(link.id, link.icon, tr(link.title), None, link.url, true, cx)),
        );
        dialog
            .title(tr("Portal.OtherAbout.Title"))
            .w(px(ABOUT_WIDTH))
            .child(body(window, rows))
            .footer(close_footer(tr("Portal.OtherAbout.Close")))
    });
}

#[cfg(test)]
mod tests {
    use super::{links, release_notes_url, version_line};

    #[test]
    fn the_release_notes_belong_to_the_running_version() {
        assert_eq!(
            release_notes_url("26.11.22.1"),
            "https://github.com/Taiizor/Sucrose/releases/tag/v26.11.22.1"
        );
    }

    #[test]
    fn the_version_line_names_the_version_once() {
        assert_eq!(version_line("26.11.22.1"), "Version: v26.11.22.1");
    }

    #[test]
    fn about_has_the_five_links_and_no_update_or_partner_row() {
        let rows = links("1.2.3.4");
        assert_eq!(rows.len(), 5);
        for row in &rows {
            assert!(row.url.starts_with("https://"), "{}", row.url);
            assert_ne!(
                rust_i18n::t!(row.title, locale = "en"),
                row.title,
                "{} has no text",
                row.title
            );
            for forbidden in ["Partners", "Update.Description"] {
                assert!(!row.title.contains(forbidden), "{}", row.title);
            }
        }
    }
}
