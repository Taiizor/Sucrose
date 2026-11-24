//! The Portal's asset source: the default component icons plus the Lucide icons the Portal's pages add. Only the added
//! icons are embedded, not the whole catalog.

use std::borrow::Cow;

use gpui_kit::{AssetSource, SharedString};

// Add the icon of a new page here (Lucide names as in `gpui_kit::assets::IconName`).
gpui_kit::assets::icon_assets!(PageIcons, [FolderX, ImageOff, Library, LibraryBig, PackageX, SearchX]);

/// The asset source the Portal registers.
pub struct PortalAssets;

impl AssetSource for PortalAssets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        match PageIcons.load(path)? {
            Some(data) => Ok(Some(data)),
            None => gpui_kit::assets::Assets.load(path),
        }
    }

    fn list(&self, path: &str) -> anyhow::Result<Vec<SharedString>> {
        let mut names = PageIcons.list(path)?;
        names.extend(gpui_kit::assets::Assets.list(path)?);
        Ok(names)
    }
}
