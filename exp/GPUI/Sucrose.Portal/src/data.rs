use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use serde_json::Value;

pub const STORE_URL: &str = "https://sucrose.soferity.com/Store";

pub fn sucrose_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("Sucrose"))
}

fn parse_json(bytes: &[u8]) -> Option<Value> {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    serde_json::from_slice(bytes).ok()
}

fn read_json(path: &Path) -> Option<Value> {
    parse_json(&fs::read(path).ok()?)
}

fn property<'a>(json: &'a Option<Value>, key: &str) -> Option<&'a Value> {
    json.as_ref()?.get("Properties")?.get(key)
}

/// Downloads a URL into memory. Only spaces are encoded, like the current Portal does.
pub fn fetch(url: &str) -> Option<Vec<u8>> {
    let mut response = ureq::get(&url.replace(' ', "%20")).call().ok()?;
    response.body_mut().read_to_vec().ok()
}

pub struct Settings {
    pub library: Option<PathBuf>,
    pub selected: Option<String>,
    pub culture: String,
    pub dark: bool,
    pub store_adult: bool,
    pub library_preview: bool,
}

impl Settings {
    pub fn load() -> Self {
        let dir = sucrose_dir();
        let setting = |name: &str| dir.as_ref().and_then(|d| read_json(&d.join("Setting").join(name)));
        let library = setting("Library.json");
        let general = setting("General.json");
        let portal = setting("Portal.json");

        Self {
            library: property(&library, "Location")
                .and_then(Value::as_str)
                .map(PathBuf::from)
                .or_else(|| dir.as_ref().map(|d| d.join("Library"))),
            selected: property(&library, "Selected").and_then(Value::as_str).map(str::to_owned),
            culture: property(&general, "Culture").and_then(Value::as_str).unwrap_or("EN").to_owned(),
            dark: property(&general, "ThemeType").and_then(Value::as_str) != Some("Light"),
            store_adult: property(&portal, "StoreAdult").and_then(Value::as_bool).unwrap_or(false),
            library_preview: property(&portal, "LibraryPreview").and_then(Value::as_bool).unwrap_or(false),
        }
    }
}

#[derive(Clone)]
pub struct Wallpaper {
    pub id: String,
    pub folder: PathBuf,
    pub title: String,
    pub description: String,
    pub kind: u64,
    pub thumbnail: PathBuf,
    pub preview: Option<PathBuf>,
    created: SystemTime,
}

pub fn load_library(location: &Path) -> Vec<Wallpaper> {
    let Ok(entries) = fs::read_dir(location) else {
        return Vec::new();
    };

    let mut list: Vec<Wallpaper> = entries
        .flatten()
        .filter_map(|entry| {
            let folder = entry.path();
            let info = read_json(&folder.join("SucroseInfo.json"))?;
            let text = |key: &str| info.get(key).and_then(Value::as_str).unwrap_or_default().to_owned();
            let preview = Some(text("Preview"))
                .filter(|name| !name.is_empty())
                .map(|name| folder.join(name))
                .filter(|path| path.is_file());

            Some(Wallpaper {
                id: entry.file_name().to_string_lossy().into_owned(),
                title: text("Title"),
                description: text("Description"),
                kind: info.get("Type").and_then(Value::as_u64).unwrap_or(0),
                thumbnail: folder.join(text("Thumbnail")),
                preview,
                created: entry.metadata().and_then(|m| m.created()).unwrap_or(SystemTime::UNIX_EPOCH),
                folder,
            })
        })
        .collect();

    list.sort_by(|a, b| b.created.cmp(&a.created));
    list
}

#[derive(Clone)]
pub struct StoreItem {
    pub key: String,
    pub category: String,
    pub source: String,
    pub cover: String,
    pub cache: PathBuf,
}

impl StoreItem {
    pub fn fallback_title(&self) -> String {
        match self.key.rsplit_once('-') {
            Some((name, suffix)) if suffix.chars().all(|c| c.is_ascii_digit()) => name.to_owned(),
            _ => self.key.clone(),
        }
    }

    pub fn cover_url(&self) -> String {
        format!("{STORE_URL}/{}/{}/{}", self.source, self.key, self.cover)
    }

    fn info_url(&self) -> String {
        format!("{STORE_URL}/{}/{}/SucroseInfo.json", self.source, self.key)
    }
}

pub struct StoreCategory {
    pub name: String,
    pub items: Vec<StoreItem>,
}

pub fn load_store(adult: bool) -> Vec<StoreCategory> {
    let Some(root) = sucrose_dir().map(|dir| dir.join("Cache").join("Store")) else {
        return Vec::new();
    };
    let Some(json) = read_json(&root.join("Store.json")) else {
        return Vec::new();
    };
    let Some(categories) = json.get("Categories").and_then(Value::as_object) else {
        return Vec::new();
    };

    categories
        .iter()
        .map(|(name, category)| {
            let items = category
                .get("Wallpapers")
                .and_then(Value::as_object)
                .map(|wallpapers| {
                    wallpapers
                        .iter()
                        .filter(|(_, w)| adult || !w.get("Adult").and_then(Value::as_bool).unwrap_or(false))
                        .map(|(key, w)| StoreItem {
                            key: key.clone(),
                            category: name.clone(),
                            source: w.get("Source").and_then(Value::as_str).unwrap_or_default().to_owned(),
                            cover: w.get("Cover").and_then(Value::as_str).unwrap_or("thumbnail.jpg").to_owned(),
                            cache: root.join(name).join(key),
                        })
                        .collect()
                })
                .unwrap_or_default();

            StoreCategory { name: name.clone(), items }
        })
        .collect()
}

/// Title and description of a store item, localized when the theme provides it.
pub fn load_store_meta(item: &StoreItem, culture: &str) -> Option<(String, String)> {
    let info = read_json(&item.cache.join("SucroseInfo.json"))
        .or_else(|| parse_json(&fetch(&item.info_url())?))?;
    let localized = info.get("Localization").and_then(|l| l.get(culture));
    let pick = |key: &str| {
        localized
            .and_then(|l| l.get(key))
            .and_then(Value::as_str)
            .or_else(|| info.get(key).and_then(Value::as_str))
            .unwrap_or_default()
            .to_owned()
    };

    Some((pick("Title"), pick("Description")))
}
