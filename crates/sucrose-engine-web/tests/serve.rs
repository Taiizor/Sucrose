//! The reply planner against the traversal vectors of the virtual origin and the Range cases (SEC-02, ENG-02, D-14): both
//! web engines stream what `serve::plan` decides, so every vector must be a plain 404 here and a Range must get the
//! reply HTTP asks for.

// A test that cannot run on this OS says so on stderr instead of passing silently.
#![allow(clippy::print_stderr)]

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU32, Ordering},
};

use sucrose_engine_web::serve::{Content, Reply, plan};

const ORIGIN: &str = "https://sucrose.wallpaper.engine";

/// A scratch folder under the temp folder, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "sucrose-serve-vectors-{name}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create scratch folder");
        Self(dir)
    }

    fn file(&self, rel: &str, body: &str) {
        let path = self.0.join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("create folders");
        fs::write(path, body).expect("write file");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Makes `link` lead to `target`: a junction on Windows (no privilege needed), a symlink elsewhere. `false` when the
/// OS refuses.
fn link_dir(link: &Path, target: &Path) -> bool {
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .output()
            .is_ok_and(|output| output.status.success())
    }
    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(target, link).is_ok()
    }
}

#[test]
fn serve_answers_every_traversal_vector_with_the_plain_refusal() {
    let outer = Scratch::new("outer");
    outer.file("secret.txt", "secret");
    outer.file("theme/index.html", "<html>");
    outer.file("theme/sub/page.html", "<p>");
    let theme = outer.0.join("theme");
    let long = format!("/{}", "a".repeat(5000));

    let vectors = [
        "/../x",
        "/../secret.txt",
        "/%2e%2e/x",
        "/%2E%2E/x",
        "/%2e./x",
        "/.%2e/x",
        "/..%2fsecret.txt",
        "/ .. /x",
        "/a/../../x",
        "/%252e%252e/x",
        "/%2500",
        "/..%5cx",
        "/a/..\\x",
        "/%5c%5cserver%5cshare",
        "//server/share",
        "/C:/Windows/win.ini",
        "/c%3a/Windows/win.ini",
        "/index.html:stream",
        "/%00",
        "/a%00.html",
        "/index.html%00.png",
        "/%ff",
        long.as_str(),
        "",
        "/",
        "?x=1",
        "/sub",
        "/sub/",
        "/secret.txt",
        "/missing.html",
    ];

    for path in vectors {
        let shown: String = path.chars().take(40).collect();
        for range in [None, Some("bytes=0-9")] {
            let reply = plan(&format!("{ORIGIN}{path}"), range, &[&theme]);
            assert_eq!(reply, Reply::not_found(), "{shown} (range {range:?})");
        }
    }
    // A foreign host is no address of the virtual origin either.
    assert_eq!(
        plan("https://example.com/index.html", None, &[&theme]),
        Reply::not_found()
    );
    // The control: the same folder still serves what is in it.
    assert_eq!(plan(&format!("{ORIGIN}/index.html"), None, &[&theme]).status, 200);
    assert_eq!(plan(&format!("{ORIGIN}/sub/page.html"), None, &[&theme]).status, 200);
}

#[test]
fn serve_refuses_a_link_inside_the_theme_that_leads_out() {
    let outer = Scratch::new("link");
    outer.file("secret.txt", "secret");
    outer.file("theme/index.html", "<html>");
    let theme = outer.0.join("theme");
    if !link_dir(&theme.join("outside"), &outer.0) {
        eprintln!("skipped: this OS refused to create a directory link");
        return;
    }

    for range in [None, Some("bytes=0-2")] {
        let reply = plan(&format!("{ORIGIN}/outside/secret.txt"), range, &[&theme]);
        assert_eq!(reply, Reply::not_found(), "range {range:?}");
    }
    assert_eq!(plan(&format!("{ORIGIN}/index.html"), None, &[&theme]).status, 200);
}

#[test]
fn serve_answers_the_range_cases_of_a_12701_byte_clip() {
    let theme = Scratch::new("range");
    let clip = fs::File::create(theme.0.join("clip.mp4")).expect("create");
    clip.set_len(12_701).expect("set length");
    let url = format!("{ORIGIN}/clip.mp4");

    let whole = plan(&url, None, &[&theme.0]);
    assert_eq!(whole.status, 200);
    assert!(matches!(
        whole.content,
        Content::File {
            start: 0,
            len: 12_701,
            ..
        }
    ));
    assert!(whole.headers.contains(&("Accept-Ranges", "bytes".to_owned())));

    let first = plan(&url, Some("bytes=0-9"), &[&theme.0]);
    assert_eq!(first.status, 206);
    assert!(matches!(first.content, Content::File { start: 0, len: 10, .. }));
    assert!(first.headers.contains(&("Content-Range", "bytes 0-9/12701".to_owned())));
    assert!(first.headers.contains(&("Accept-Ranges", "bytes".to_owned())));

    let past = plan(&url, Some("bytes=20000-"), &[&theme.0]);
    assert_eq!(past.status, 416);
    assert_eq!(past.content, Content::Text(""));
    assert!(past.headers.contains(&("Content-Range", "bytes */12701".to_owned())));
}
