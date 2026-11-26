//! `cargo xtask localize`: the legacy XAML string resources converted to rust-i18n YAML (D-015, D-20).
//!
//! The legacy Portal keeps its texts in `src/Library/Sucrose.Resources/Locales/Portal/**/Portal.*.EN.xaml` and in the
//! language-neutral `Portal.xaml`. The Portal crate needs the same keys in `apps/sucrose-portal/locales/legacy.<lang>.yml`.
//! The legacy files are only read, never changed.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::workspace_root;

/// Folder of the legacy Portal resources, relative to the workspace root.
const XAML_ROOT: &str = "src/Library/Sucrose.Resources/Locales/Portal";
/// Folder of the Portal's locale files, relative to the workspace root.
const OUT_DIR: &str = "apps/sucrose-portal/locales";
/// Key prefixes of the UI-SPEC String Inventory; every other legacy key (store, wallpaper cycling, ...) stays out of Phase 3.
const PREFIXES: [&str; 14] = [
    "Portal.MainWindow.",
    "Portal.EmptyLibraryPage.",
    "Portal.SearchLibraryPage.",
    "Portal.LibraryCard.",
    "Portal.Pagination.",
    "Portal.TrayIcon.",
    "Portal.OtherHelp.",
    "Portal.OtherAbout.",
    "Portal.GeneralSettingPage.",
    "Portal.PersonalSettingPage.",
    "Portal.WallpaperSettingPage.",
    "Portal.Area.",
    "Portal.Enum.",
    "Portal.Quoting",
];
/// Single keys of the String Inventory that have no common prefix.
const EXACT_KEYS: [&str; 2] = ["Portal.Release", "Portal.Copyright"];
/// Legacy keys whose English text Phase 3 words differently (the corrupt themes go to the system bin, not away for good).
/// They are left out of the generated file and defined in the Portal's hand-written `sucrose.en.yml` instead, so no key is
/// defined in two files.
const REPLACED_KEYS: [&str; 2] = [
    "Portal.PersonalSettingPage.Delete.Description",
    "Portal.PersonalSettingPage.Delete.DeleteHint",
];
/// What `--check` prints when the committed file is out of date.
const STALE: &str = "stale: run cargo xtask localize --lang EN";

/// Extracts every `system:String` entry of a XAML resource dictionary as (key, decoded text), in file order.
fn parse_xaml(xaml: &str) -> Vec<(String, String)> {
    const TAG: &str = "<system:String";
    const CLOSE: &str = "</system:String>";
    let mut entries = Vec::new();
    let mut rest = xaml;
    while let Some(start) = rest.find('<') {
        rest = &rest[start..];
        if let Some(comment) = rest.strip_prefix("<!--") {
            rest = comment.find("-->").map_or("", |end| &comment[end + 3..]);
            continue;
        }
        let Some(after_tag) = rest.strip_prefix(TAG) else {
            rest = &rest[1..];
            continue;
        };
        // `<system:StringList` and friends are other elements.
        if !after_tag.starts_with(|c: char| c.is_whitespace() || c == '>' || c == '/') {
            rest = after_tag;
            continue;
        }
        let Some(tag_end) = start_tag_end(after_tag) else {
            break;
        };
        let attributes = &after_tag[..tag_end];
        let body = &after_tag[tag_end + 1..];
        if attributes.ends_with('/') {
            rest = body;
            continue;
        }
        let Some(close) = body.find(CLOSE) else {
            break;
        };
        if let Some(key) = attribute(attributes, "x:Key") {
            entries.push((key, decode_entities(&body[..close])));
        }
        rest = &body[close + CLOSE.len()..];
    }
    entries
}

/// Byte index of the `>` that ends a start tag, ignoring one inside an attribute value.
fn start_tag_end(tag: &str) -> Option<usize> {
    let mut quote = None;
    for (index, c) in tag.char_indices() {
        match (quote, c) {
            (None, '"' | '\'') => quote = Some(c),
            (Some(open), _) if open == c => quote = None,
            (None, '>') => return Some(index),
            _ => {}
        }
    }
    None
}

/// The decoded value of a quoted attribute in the text between a tag name and its `>`.
fn attribute(attributes: &str, name: &str) -> Option<String> {
    let mut from = 0;
    while let Some(found) = attributes[from..].find(name) {
        let at = from + found;
        from = at + name.len();
        if !attributes[..at].chars().next_back().is_none_or(char::is_whitespace) {
            continue;
        }
        let Some(value) = attributes[from..].trim_start().strip_prefix('=') else {
            continue;
        };
        let value = value.trim_start();
        let quote = value.chars().next().filter(|c| matches!(c, '"' | '\''))?;
        let value = &value[1..];
        return value.find(quote).map(|end| decode_entities(&value[..end]));
    }
    None
}

/// Decodes the XML entities XAML text may contain (named and numeric).
fn decode_entities(text: &str) -> String {
    let mut decoded = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(amp) = rest.find('&') {
        decoded.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let reference = rest
            .find(';')
            .and_then(|semi| decode_reference(&rest[1..semi]).map(|c| (c, semi)));
        if let Some((c, semi)) = reference {
            decoded.push(c);
            rest = &rest[semi + 1..];
        } else {
            // Not a reference this converter knows: keep the ampersand as it is.
            decoded.push('&');
            rest = &rest[1..];
        }
    }
    decoded.push_str(rest);
    decoded
}

/// The character a named or numeric entity (the text between `&` and `;`) stands for.
fn decode_reference(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => {
            let digits = name.strip_prefix('#')?;
            let code = match digits.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => digits.parse().ok()?,
            };
            char::from_u32(code)
        }
    }
}

/// Whether a legacy key belongs to the String Inventory.
fn keep_key(key: &str) -> bool {
    if REPLACED_KEYS.contains(&key) {
        return false;
    }
    PREFIXES.iter().any(|prefix| key.starts_with(prefix)) || EXACT_KEYS.contains(&key)
}

/// The YAML file text for the given entries: a header comment, then sorted flat dotted keys as double-quoted scalars.
fn to_yaml(entries: &BTreeMap<String, String>) -> String {
    let mut yaml = String::from(
        "# Generated by `cargo xtask localize` from the legacy XAML resources. Do not edit: run the task again.\n",
    );
    for (key, value) in entries {
        yaml.push_str(&quote(key));
        yaml.push_str(": ");
        yaml.push_str(&quote(value));
        yaml.push('\n');
    }
    yaml
}

/// A YAML double-quoted scalar.
fn quote(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');
    for c in text.chars() {
        match c {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            c if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}') => {
                quoted.push_str(&format!("\\u{:04X}", u32::from(c)));
            }
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

/// Collects the English and language-neutral XAML files below `dir`, in a stable order.
fn collect_files(dir: &Path, lang: &str, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<_, _>>()?;
    entries.sort();
    let suffix = format!(".{lang}.xaml");
    for path in entries {
        if path.is_dir() {
            collect_files(&path, lang, out)?;
            continue;
        }
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        if name == "Portal.xaml" || name.ends_with(&suffix) {
            out.push(path);
        }
    }
    Ok(())
}

/// Reads the legacy resources of one language and builds the YAML text.
fn generate(root: &Path, lang: &str) -> Result<(String, usize), String> {
    let mut files = Vec::new();
    collect_files(&root.join(XAML_ROOT), lang, &mut files)
        .map_err(|error| format!("cannot read {XAML_ROOT}: {error}"))?;
    let mut entries = BTreeMap::new();
    for file in &files {
        let text = fs::read_to_string(file).map_err(|error| format!("cannot read {}: {error}", file.display()))?;
        for (key, value) in parse_xaml(text.trim_start_matches('\u{feff}')) {
            if !keep_key(&key) {
                continue;
            }
            if let Some(previous) = entries.insert(key.clone(), value.clone())
                && previous != value
            {
                return Err(format!(
                    "{key} is defined twice with different texts ({})",
                    file.display()
                ));
            }
        }
    }
    let count = entries.len();
    Ok((to_yaml(&entries), count))
}

/// `cargo xtask localize --lang EN [--check]`
pub fn run(args: &[String]) -> ExitCode {
    match run_inner(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run_inner(args: &[String]) -> Result<(), String> {
    let mut lang = None;
    let mut check = false;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--lang" => lang = Some(it.next().ok_or("--lang needs a value")?.to_uppercase()),
            "--check" => check = true,
            other => return Err(format!("unknown argument '{other}'")),
        }
    }
    let lang = lang.ok_or("give --lang EN")?;
    if lang != "EN" {
        return Err(format!(
            "only --lang EN is supported in Phase 3; the other 24 languages arrive with UI-06 (Phase 5), not '{lang}'"
        ));
    }
    let root = workspace_root();
    let (yaml, count) = generate(&root, &lang)?;
    let target = root.join(OUT_DIR).join(format!("legacy.{}.yml", lang.to_lowercase()));
    if check {
        // A Windows checkout may have converted the file to CRLF; the content is what counts.
        let current = fs::read_to_string(&target).unwrap_or_default().replace("\r\n", "\n");
        if current != yaml {
            return Err(STALE.to_owned());
        }
        println!("{} is current ({count} keys)", target.display());
        return Ok(());
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    fs::write(&target, yaml).map_err(|error| format!("cannot write {}: {error}", target.display()))?;
    println!("wrote {count} keys to {}", target.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "<ResourceDictionary xmlns=\"http://schemas.microsoft.com/winfx/2006/xaml/presentation\"\n    xmlns:x=\"http://schemas.microsoft.com/winfx/2006/xaml\"\n    xmlns:system=\"clr-namespace:System;assembly=mscorlib\">\n";

    fn pair(key: &str, text: &str) -> (String, String) {
        (key.to_owned(), text.to_owned())
    }

    #[test]
    fn parse_xaml_returns_entries_in_file_order() {
        let xaml = format!(
            "{HEADER}    <system:String x:Key=\"Portal.B\">Second</system:String>\n\n    <system:String x:Key=\"Portal.A\">First one</system:String>\n</ResourceDictionary>"
        );
        assert_eq!(
            parse_xaml(&xaml),
            vec![pair("Portal.B", "Second"), pair("Portal.A", "First one")]
        );
    }

    #[test]
    fn parse_xaml_decodes_entities_in_the_text() {
        let xaml = format!(
            "{HEADER}<system:String x:Key=\"K\">Appearance &amp; Behavior &lt;b&gt; &quot;q&quot; it&apos;s &#x2026; &#8230;</system:String>"
        );
        assert_eq!(
            parse_xaml(&xaml),
            vec![pair("K", "Appearance & Behavior <b> \"q\" it's \u{2026} \u{2026}")]
        );
    }

    #[test]
    fn decode_entities_keeps_unknown_and_unterminated_references() {
        assert_eq!(decode_entities("a &unknown; b &amp"), "a &unknown; b &amp");
        assert_eq!(decode_entities("&amp;amp;"), "&amp;");
    }

    #[test]
    fn parse_xaml_skips_self_closing_other_elements_and_comments() {
        let xaml = format!(
            "{HEADER}<!-- <system:String x:Key=\"Commented\">no</system:String> -->\n<system:String x:Key=\"Empty\" />\n<system:Boolean x:Key=\"Flag\">True</system:Boolean>\n<system:StringList x:Key=\"Other\">x</system:StringList>\n<system:String x:Key=\"Kept\">yes</system:String>"
        );
        assert_eq!(parse_xaml(&xaml), vec![pair("Kept", "yes")]);
    }

    #[test]
    fn to_yaml_sorts_keys_and_escapes_scalars() {
        let mut entries = BTreeMap::new();
        entries.insert("Portal.B".to_owned(), "say \"hi\" \\ now\nnext\ttab".to_owned());
        entries.insert("Portal.A".to_owned(), "caf\u{e9} \u{a9}".to_owned());
        let yaml = to_yaml(&entries);
        assert!(yaml.starts_with("# Generated by `cargo xtask localize"));
        assert!(!yaml.contains('\r'));
        let body: Vec<&str> = yaml.lines().filter(|line| !line.starts_with('#')).collect();
        assert_eq!(
            body,
            vec![
                "\"Portal.A\": \"caf\u{e9} \u{a9}\"",
                "\"Portal.B\": \"say \\\"hi\\\" \\\\ now\\nnext\\ttab\"",
            ]
        );
    }

    #[test]
    fn keep_key_drops_the_keys_phase_3_rewords() {
        assert!(keep_key("Portal.PersonalSettingPage.Delete"));
        assert!(!keep_key("Portal.PersonalSettingPage.Delete.Description"));
        assert!(!keep_key("Portal.PersonalSettingPage.Delete.DeleteHint"));
    }

    #[test]
    fn keep_key_keeps_the_inventory_prefixes_only() {
        for key in [
            "Portal.MainWindow.Title",
            "Portal.GeneralSettingPage.WindowBackdrop.Title",
            "Portal.Enum.SortMode.Name",
            "Portal.Quoting0",
            "Portal.Quoting85",
            "Portal.Release",
            "Portal.Copyright",
        ] {
            assert!(keep_key(key), "{key} should be kept");
        }
        for key in [
            "Portal.StorePage.Title",
            "Portal.LibraryPage.Title",
            "Portal.ReleaseNotes",
            "Portal.WallpaperCycling.Title",
            "Other.Quoting0",
        ] {
            assert!(!keep_key(key), "{key} should be dropped");
        }
    }
}
