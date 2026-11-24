//! The Help dialog (UI-05): five link rows to the pages of the project. Each row opens its link in the OS browser through
//! [`super::open_link`], which calls GPUI's `App::open_url`.

use gpui_kit::assets::IconName;
use gpui_kit::component::WindowExt as _;
use gpui_kit::*;

use super::{DIALOG_WIDTH, body, close_footer, link_row, urls};
use crate::i18n::tr;

/// One row of the dialog: its element id, icon, the keys of its title and description and its target.
struct HelpLink {
    id: &'static str,
    icon: IconName,
    title: &'static str,
    description: &'static str,
    url: &'static str,
}

/// The rows, in the legacy order.
const LINKS: [HelpLink; 5] = [
    HelpLink {
        id: "help-web-page",
        icon: IconName::Globe,
        title: "Portal.OtherHelp.WebPage",
        description: "Portal.OtherHelp.WebPage.Description",
        url: urls::OFFICIAL,
    },
    HelpLink {
        id: "help-documentation",
        icon: IconName::BookOpen,
        title: "Portal.OtherHelp.Documentation",
        description: "Portal.OtherHelp.Documentation.Description",
        url: urls::DOCUMENTATION,
    },
    HelpLink {
        id: "help-repository",
        icon: IconName::Code,
        title: "Portal.OtherHelp.Repository",
        description: "Portal.OtherHelp.Repository.Description",
        url: urls::REPOSITORY,
    },
    HelpLink {
        id: "help-discussions",
        icon: IconName::MessagesSquare,
        title: "Portal.OtherHelp.Discussions",
        description: "Portal.OtherHelp.Discussions.Description",
        url: urls::DISCUSSIONS,
    },
    HelpLink {
        id: "help-report",
        icon: IconName::Bug,
        title: "Portal.OtherHelp.Report",
        description: "Portal.OtherHelp.Report.Description",
        url: urls::REPORT,
    },
];

/// Opens the Help dialog.
pub fn open(window: &mut Window, cx: &mut App) {
    window.open_dialog(cx, |dialog, window, cx| {
        let rows = LINKS.iter().map(|link| {
            link_row(
                link.id,
                link.icon,
                tr(link.title),
                Some(tr(link.description)),
                link.url.to_owned(),
                false,
                cx,
            )
        });
        dialog
            .title(tr("Portal.OtherHelp.Title"))
            .w(px(DIALOG_WIDTH))
            .child(body(window, rows))
            .footer(close_footer(tr("Portal.OtherHelp.Close")))
    });
}

#[cfg(test)]
mod tests {
    use super::LINKS;

    #[test]
    fn help_has_the_five_legacy_rows_with_distinct_targets() {
        assert_eq!(LINKS.len(), 5);
        let mut urls: Vec<&str> = LINKS.iter().map(|link| link.url).collect();
        urls.sort_unstable();
        urls.dedup();
        assert_eq!(urls.len(), 5);
        let mut ids: Vec<&str> = LINKS.iter().map(|link| link.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 5, "element ids are unique");
    }

    #[test]
    fn every_row_has_text_for_its_title_and_description() {
        for link in &LINKS {
            for key in [link.title, link.description] {
                assert_ne!(rust_i18n::t!(key, locale = "en"), key, "{key} has no text");
            }
        }
    }
}
