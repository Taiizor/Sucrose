//! The traversal vectors of the virtual origin (SEC-02, D-14): every spelling of a way out of the theme folder must be
//! refused with its own reason, and a link that leads out must not be followed.

// A test that cannot run on this OS says so on stderr instead of passing silently.
#![allow(clippy::print_stderr)]

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU32, Ordering},
};

use sucrose_engine_web::origin::{Reject, resolve};

/// A scratch folder under the temp folder, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "sucrose-vectors-{name}-{}-{}",
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
fn origin_refuses_every_traversal_vector_with_its_reason() {
    let outer = Scratch::new("outer");
    outer.file("secret.txt", "secret");
    outer.file("theme/index.html", "<html>");
    outer.file("theme/sub/page.html", "<p>");
    let theme = outer.0.join("theme");
    let long = format!("/{}", "a".repeat(5000));

    let vectors: Vec<(&str, Reject)> = vec![
        ("/../x", Reject::Dots),
        ("/../secret.txt", Reject::Dots),
        ("/%2e%2e/x", Reject::Dots),
        ("/%2E%2E/x", Reject::Dots),
        ("/%2e./x", Reject::Dots),
        ("/.%2e/x", Reject::Dots),
        ("/..%2fsecret.txt", Reject::Dots),
        ("/ .. /x", Reject::Dots),
        ("/a/../../x", Reject::Dots),
        ("/%252e%252e/x", Reject::DoubleEncoded),
        ("/%2500", Reject::DoubleEncoded),
        ("/..%5cx", Reject::Backslash),
        ("/a/..\\x", Reject::Backslash),
        ("/%5c%5cserver%5cshare", Reject::Backslash),
        ("//server/share", Reject::Unc),
        ("/C:/Windows/win.ini", Reject::DriveOrColon),
        ("/c%3a/Windows/win.ini", Reject::DriveOrColon),
        ("/index.html:stream", Reject::DriveOrColon),
        ("/%00", Reject::Nul),
        ("/a%00.html", Reject::Nul),
        ("/index.html%00.png", Reject::Nul),
        ("/%ff", Reject::BadUtf8),
        (long.as_str(), Reject::TooLong),
        ("", Reject::Empty),
        ("/", Reject::Empty),
        ("?x=1", Reject::Empty),
        ("/sub", Reject::NotAFile),
        ("/sub/", Reject::NotAFile),
        ("/secret.txt", Reject::NotAFile),
        ("/missing.html", Reject::NotAFile),
    ];

    for (path, expected) in vectors {
        let shown: String = path.chars().take(40).collect();
        assert_eq!(resolve(path, &[&theme]), Err(expected), "{shown}");
    }
    // The control: the same folder still serves what is in it.
    assert!(resolve("/index.html", &[&theme]).is_ok());
    assert!(resolve("/sub/page.html", &[&theme]).is_ok());
}

#[test]
fn origin_refuses_a_link_inside_the_theme_that_leads_out() {
    let outer = Scratch::new("link");
    outer.file("secret.txt", "secret");
    outer.file("theme/index.html", "<html>");
    let theme = outer.0.join("theme");
    if !link_dir(&theme.join("outside"), &outer.0) {
        eprintln!("skipped: this OS refused to create a directory link");
        return;
    }

    assert_eq!(resolve("/outside/secret.txt", &[&theme]), Err(Reject::Outside));
    assert!(resolve("/index.html", &[&theme]).is_ok());
}

#[test]
fn origin_serves_the_content_folder_when_the_theme_link_leads_out() {
    let outer = Scratch::new("fallback");
    outer.file("elsewhere/page.html", "elsewhere");
    outer.file("theme/index.html", "<html>");
    outer.file("content/page.html", "content");
    let theme = outer.0.join("theme");
    let content = outer.0.join("content");
    if !link_dir(&theme.join("page.html.d"), &outer.0.join("elsewhere")) {
        eprintln!("skipped: this OS refused to create a directory link");
        return;
    }

    // A link out of the theme is refused there, and the lookup goes on to the content folder without ever
    // serving the file the link points to.
    assert_eq!(resolve("/page.html.d/page.html", &[&theme]), Err(Reject::Outside));
    assert_eq!(
        resolve("/page.html.d/page.html", &[&theme, &content]),
        Err(Reject::Outside)
    );
    let served = resolve("/page.html", &[&theme, &content]).expect("content page");
    assert_eq!(served.root, 1);
}

#[test]
fn origin_resolves_parallel_requests_independently() {
    let outer = Scratch::new("parallel");
    for index in 0..8 {
        outer.file(&format!("theme/f{index}.txt"), &"x".repeat(index + 1));
    }
    let theme = outer.0.join("theme");

    std::thread::scope(|scope| {
        for index in 0..8usize {
            let theme = &theme;
            scope.spawn(move || {
                for _ in 0..50 {
                    let file = resolve(&format!("/f{index}.txt"), &[theme]).expect("resolves");
                    assert_eq!(file.len, index as u64 + 1);
                    assert_eq!(resolve("/%2e%2e/x", &[theme]), Err(Reject::Dots));
                }
            });
        }
    });
}
