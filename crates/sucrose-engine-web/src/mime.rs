//! The fixed MIME table of the virtual origin (legacy parity with the C# Watson server, plus the module and WebAssembly
//! types a page needs). No MIME crate: a theme folder is ordinary files and the table is short.

/// The type of a file with an unknown extension.
pub const DEFAULT: &str = "application/octet-stream";

/// Extension and type; the first block is the legacy table, the second what modern pages need on top of it.
const TABLE: &[(&str, &str)] = &[
    ("rar", "application/x-rar-compressed"),
    ("xhtml", "application/xhtml+xml"),
    ("js", "application/javascript"),
    ("hdr", "image/vnd.radiance"),
    ("tar", "application/x-tar"),
    ("json", "application/json"),
    ("glb", "model/gltf-binary"),
    ("gltf", "model/gltf+json"),
    ("zip", "application/zip"),
    ("xml", "application/xml"),
    ("svg", "image/svg+xml"),
    ("woff2", "font/woff2"),
    ("md", "text/markdown"),
    ("ico", "image/x-icon"),
    ("webp", "image/webp"),
    ("webm", "video/webm"),
    ("tiff", "image/tiff"),
    ("shtml", "text/html"),
    ("jpeg", "image/jpeg"),
    ("woff", "font/woff"),
    ("txt", "text/plain"),
    ("jpg", "image/jpeg"),
    ("mp3", "audio/mpeg"),
    ("html", "text/html"),
    ("wav", "audio/wav"),
    ("png", "image/png"),
    ("ogg", "audio/ogg"),
    ("mp4", "video/mp4"),
    ("htm", "text/html"),
    ("gif", "image/gif"),
    ("bmp", "image/bmp"),
    ("ttf", "font/ttf"),
    ("otf", "font/otf"),
    ("csv", "text/csv"),
    ("css", "text/css"),
    // Not in the legacy table (it answered octet-stream, which breaks module scripts and streaming compilation).
    ("mjs", "application/javascript"),
    ("wasm", "application/wasm"),
];

/// The MIME type for an extension, with or without the leading dot, in any case.
#[must_use]
pub fn mime_for(ext: &str) -> &'static str {
    let ext = ext.strip_prefix('.').unwrap_or(ext);
    TABLE
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(ext))
        .map_or(DEFAULT, |(_, mime)| *mime)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_legacy_entries_keep_their_types() {
        for (ext, mime) in [
            ("html", "text/html"),
            ("htm", "text/html"),
            ("js", "application/javascript"),
            ("json", "application/json"),
            ("css", "text/css"),
            ("png", "image/png"),
            ("jpg", "image/jpeg"),
            ("jpeg", "image/jpeg"),
            ("gif", "image/gif"),
            ("webp", "image/webp"),
            ("svg", "image/svg+xml"),
            ("ico", "image/x-icon"),
            ("mp4", "video/mp4"),
            ("webm", "video/webm"),
            ("ogg", "audio/ogg"),
            ("mp3", "audio/mpeg"),
            ("wav", "audio/wav"),
            ("woff", "font/woff"),
            ("woff2", "font/woff2"),
            ("ttf", "font/ttf"),
            ("otf", "font/otf"),
            ("txt", "text/plain"),
            ("xml", "application/xml"),
            ("glb", "model/gltf-binary"),
            ("gltf", "model/gltf+json"),
            ("hdr", "image/vnd.radiance"),
            ("csv", "text/csv"),
        ] {
            assert_eq!(mime_for(ext), mime, "{ext}");
        }
    }

    #[test]
    fn modules_and_webassembly_get_their_types() {
        assert_eq!(mime_for("mjs"), "application/javascript");
        assert_eq!(mime_for("wasm"), "application/wasm");
    }

    #[test]
    fn the_extension_is_matched_in_any_case_and_with_a_dot() {
        assert_eq!(mime_for("HTML"), "text/html");
        assert_eq!(mime_for(".Png"), "image/png");
    }

    #[test]
    fn an_unknown_extension_is_octet_stream() {
        assert_eq!(mime_for("xyz"), DEFAULT);
        assert_eq!(mime_for(""), DEFAULT);
    }
}
