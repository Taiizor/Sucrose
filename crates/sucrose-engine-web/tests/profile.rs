//! The profile rule of D-33 and D-35 as amended by 04-37 D9: the instance that asks for the persistent profile keeps its
//! web data in `<web>/<engine>`, every other one runs on an emptied ephemeral folder, and the OS lock guards that no two
//! instances share the persistent one, across processes too.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU32, Ordering},
    thread,
    time::{Duration, Instant},
};

use sucrose_engine_web::{
    profile::{ProfileKind, acquire},
    serve::{Content, plan},
};

/// How long an instance beside a lock holder of another process waits before it runs ephemeral.
const HOLDER_WAIT: Duration = Duration::from_millis(100);

/// A scratch folder under the temp folder, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "sucrose-profile-{name}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create scratch folder");
        Self(dir)
    }

    fn web(&self) -> PathBuf {
        self.0.join("Web")
    }

    fn ephemeral(&self) -> PathBuf {
        self.0.join("Cache").join("WebEphemeral")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn profile_first_acquire_is_persistent_and_a_concurrent_one_is_ephemeral() {
    let scratch = Scratch::new("first");

    let first = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        0,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("first");
    let second = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        1,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("second");

    assert_eq!(first.kind, ProfileKind::Persistent);
    assert_eq!(first.path, scratch.web().join("Cef"));
    assert!(first.path.is_dir());
    assert_eq!(second.kind, ProfileKind::Ephemeral);
    assert_eq!(second.path, scratch.ephemeral().join("Cef").join("1"));
    assert!(second.path.is_dir());
}

#[test]
fn profile_an_ephemeral_request_never_takes_the_lock() {
    let scratch = Scratch::new("ephemeral-asks");

    let ephemeral = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        1,
        ProfileKind::Ephemeral,
        Duration::ZERO,
    )
    .expect("ephemeral");
    let persistent = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        0,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("persistent");

    assert_eq!(ephemeral.kind, ProfileKind::Ephemeral);
    assert_eq!(ephemeral.path, scratch.ephemeral().join("Cef").join("1"));
    assert!(ephemeral.path.is_dir());
    assert_eq!(
        persistent.kind,
        ProfileKind::Persistent,
        "the ephemeral request left the lock free"
    );
}

#[test]
fn profile_an_ephemeral_request_does_not_create_the_persistent_folder() {
    let scratch = Scratch::new("ephemeral-no-folder");

    let lease = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        1,
        ProfileKind::Ephemeral,
        Duration::ZERO,
    )
    .expect("ephemeral");

    assert_eq!(lease.kind, ProfileKind::Ephemeral);
    assert!(
        !scratch.web().exists(),
        "no persistent folder and no lock file were created"
    );
}

#[test]
fn profile_a_persistent_request_waits_for_the_bound_and_then_runs_ephemeral() {
    let scratch = Scratch::new("wait-bound");
    let _holder = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        0,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("holder");

    let started = Instant::now();
    let lease = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        1,
        ProfileKind::Persistent,
        Duration::from_millis(200),
    )
    .expect("waiter");
    let waited = started.elapsed();

    assert_eq!(lease.kind, ProfileKind::Ephemeral);
    assert!(waited >= Duration::from_millis(200), "waited {waited:?}");
    assert!(waited < Duration::from_secs(2), "waited {waited:?}");
}

#[test]
fn profile_a_persistent_request_takes_the_lock_a_holder_drops_during_the_wait() {
    let scratch = Scratch::new("wait-release");
    let holder = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        0,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("holder");

    let lease = thread::scope(|scope| {
        scope.spawn(move || {
            thread::sleep(Duration::from_millis(100));
            drop(holder);
        });
        acquire(
            &scratch.web(),
            "Cef",
            &scratch.ephemeral(),
            1,
            ProfileKind::Persistent,
            Duration::from_secs(2),
        )
        .expect("waiter")
    });

    assert_eq!(lease.kind, ProfileKind::Persistent);
}

#[test]
fn profile_is_persistent_again_after_the_first_lease_is_dropped() {
    let scratch = Scratch::new("again");
    let first = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        0,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("first");
    assert_eq!(first.kind, ProfileKind::Persistent);
    drop(first);

    let next = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        0,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("next");

    assert_eq!(next.kind, ProfileKind::Persistent);
}

#[test]
fn profile_persistent_folder_keeps_its_files_across_leases() {
    let scratch = Scratch::new("keeps");
    let first = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        0,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("first");
    fs::write(first.path.join("Local Storage"), b"counter=1").expect("write web data");
    drop(first);

    let next = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        0,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("next");

    assert_eq!(next.kind, ProfileKind::Persistent);
    assert_eq!(fs::read(next.path.join("Local Storage")).expect("read"), b"counter=1");
}

#[test]
fn profile_ephemeral_folder_from_an_earlier_run_is_emptied_by_acquire() {
    let scratch = Scratch::new("wipe");
    let stale = scratch.ephemeral().join("Cef").join("1");
    fs::create_dir_all(stale.join("nested")).expect("stale folder");
    fs::write(stale.join("old.txt"), b"old").expect("stale file");
    fs::write(stale.join("nested").join("deep.txt"), b"old").expect("stale nested file");
    let _holder = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        0,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("holder");

    let lease = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        1,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("ephemeral");

    assert_eq!(lease.kind, ProfileKind::Ephemeral);
    assert_eq!(lease.path, stale);
    assert_eq!(
        fs::read_dir(&lease.path).expect("read").count(),
        0,
        "the old files are gone"
    );
}

#[test]
fn profile_engines_do_not_share_a_lock() {
    let scratch = Scratch::new("engines");

    let cef = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        0,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("cef");
    let webview2 = acquire(
        &scratch.web(),
        "WebView2",
        &scratch.ephemeral(),
        0,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("webview2");

    assert_eq!(cef.kind, ProfileKind::Persistent);
    assert_eq!(webview2.kind, ProfileKind::Persistent);
    assert_eq!(webview2.path, scratch.web().join("WebView2"));
}

#[test]
fn profile_engine_name_cannot_leave_the_web_folder() {
    let scratch = Scratch::new("name");

    for engine in ["", "..", "a/b", "a\\b", "../x"] {
        let error = acquire(
            &scratch.web(),
            engine,
            &scratch.ephemeral(),
            0,
            ProfileKind::Persistent,
            Duration::ZERO,
        )
        .expect_err(engine);
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput, "{engine:?}");
    }
}

/// The environment variable that turns [`profile_holder_child`] into a lock holder.
const HOLD_ENV: &str = "SUCROSE_PROFILE_HOLD_DIR";

/// The engine name the lock holder takes the profile of; `Cef` when it is not set.
const HOLD_ENGINE_ENV: &str = "SUCROSE_PROFILE_HOLD_ENGINE";

/// Not a test of its own: when the parent test starts this binary with [`HOLD_ENV`] set it takes the persistent
/// profile, says so with a marker file and holds it until it is killed (or a minute passed, so it never lingers).
#[test]
fn profile_holder_child() {
    let Some(dir) = std::env::var_os(HOLD_ENV) else {
        return;
    };
    let dir = PathBuf::from(dir);
    let engine = std::env::var(HOLD_ENGINE_ENV).unwrap_or_else(|_| "Cef".to_owned());
    let lease = acquire(
        &dir.join("Web"),
        &engine,
        &dir.join("Cache").join("WebEphemeral"),
        0,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("child acquire");
    assert_eq!(
        lease.kind,
        ProfileKind::Persistent,
        "the child must hold the persistent profile"
    );
    fs::write(dir.join("ready"), b"").expect("ready marker");
    let deadline = Instant::now() + Duration::from_secs(60);
    while Instant::now() < deadline {
        thread::sleep(Duration::from_millis(50));
    }
}

fn wait_for(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !path.exists() {
        assert!(Instant::now() < deadline, "{} did not appear", path.display());
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn profile_lock_held_by_another_process_makes_acquire_ephemeral_until_it_dies() {
    let scratch = Scratch::new("process");
    let exe = std::env::current_exe().expect("the test binary has a path");
    let mut child = Command::new(exe)
        .args(["--exact", "profile_holder_child", "--nocapture", "--test-threads=1"])
        .env(HOLD_ENV, &scratch.0)
        .spawn()
        .expect("start the holder process");
    wait_for(&scratch.0.join("ready"));

    let beside = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        2,
        ProfileKind::Persistent,
        HOLDER_WAIT,
    )
    .expect("beside the holder");
    let beside_kind = beside.kind;
    child.kill().expect("kill the holder");
    child.wait().expect("reap the holder");
    drop(beside);
    let after = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        2,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("after the holder died");

    assert_eq!(beside_kind, ProfileKind::Ephemeral);
    assert_eq!(
        after.kind,
        ProfileKind::Persistent,
        "a dead process's lock is released with it"
    );
}

/// Two WebView2 instances never share the persistent profile (ENG-02, D-33): while one process holds it a second
/// instance gets an emptied ephemeral folder of its own slot, two instances beside the holder do not collide with each
/// other, and the holder's death releases the lock for the next start.
#[test]
fn concurrent_webview2_instances_never_share_the_persistent_profile() {
    let scratch = Scratch::new("webview2");
    let exe = std::env::current_exe().expect("the test binary has a path");
    let mut child = Command::new(exe)
        .args(["--exact", "profile_holder_child", "--nocapture", "--test-threads=1"])
        .env(HOLD_ENV, &scratch.0)
        .env(HOLD_ENGINE_ENV, "WebView2")
        .spawn()
        .expect("start the holder process");
    wait_for(&scratch.0.join("ready"));

    let beside = acquire(
        &scratch.web(),
        "WebView2",
        &scratch.ephemeral(),
        1,
        ProfileKind::Persistent,
        HOLDER_WAIT,
    )
    .expect("beside the holder");
    let further = acquire(
        &scratch.web(),
        "WebView2",
        &scratch.ephemeral(),
        2,
        ProfileKind::Persistent,
        HOLDER_WAIT,
    )
    .expect("further beside the holder");
    let cef = acquire(
        &scratch.web(),
        "Cef",
        &scratch.ephemeral(),
        0,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("cef beside the WebView2 holder");
    let (beside_kind, beside_path) = (beside.kind, beside.path.clone());
    let (further_kind, further_path) = (further.kind, further.path.clone());
    child.kill().expect("kill the holder");
    child.wait().expect("reap the holder");
    drop((beside, further));
    let after = acquire(
        &scratch.web(),
        "WebView2",
        &scratch.ephemeral(),
        1,
        ProfileKind::Persistent,
        Duration::ZERO,
    )
    .expect("after the holder died");

    assert_eq!(beside_kind, ProfileKind::Ephemeral);
    assert_eq!(beside_path, scratch.ephemeral().join("WebView2").join("1"));
    assert_eq!(further_kind, ProfileKind::Ephemeral);
    assert_eq!(further_path, scratch.ephemeral().join("WebView2").join("2"));
    assert_ne!(beside_path, further_path, "no two instances share a folder");
    assert_eq!(
        cef.kind,
        ProfileKind::Persistent,
        "the engines keep their own persistent folder and lock"
    );
    assert_eq!(
        after.kind,
        ProfileKind::Persistent,
        "a killed holder's lock is released for the next start"
    );
    assert_eq!(after.path, scratch.web().join("WebView2"));
}

/// The reply planner holds no state (ENG-02): two instances planning at once for different theme folders each get the
/// answer for their own root, never the other's.
#[test]
fn concurrent_instances_planning_origin_replies_get_their_own_answers() {
    let first = Scratch::new("plan-first");
    let second = Scratch::new("plan-second");
    fs::write(first.0.join("index.html"), "1").expect("first page");
    fs::write(second.0.join("index.html"), "22").expect("second page");
    let url = "https://sucrose.wallpaper.engine/index.html";

    thread::scope(|scope| {
        for (root, expected) in [(&first.0, 1_u64), (&second.0, 2_u64)] {
            for _ in 0..4 {
                scope.spawn(move || {
                    for _ in 0..200 {
                        let reply = plan(url, None, &[root.as_path()]);
                        assert_eq!(reply.status, 200);
                        let Content::File { len, .. } = reply.content else {
                            panic!("a theme page is served from its file");
                        };
                        assert_eq!(len, expected, "the answer belongs to this instance's root");
                    }
                });
            }
        }
    });
}

/// A file held open without delete sharing makes the removal of its folder fail like a process of the ended engine
/// still using it (sharing violation, os error 32).
#[cfg(windows)]
mod held_folder {
    use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt, thread, time::Duration};

    use super::{Instant, ProfileKind, Scratch, acquire, fs};

    /// Holds `<ephemeral>/Cef/1/held.bin` open for `hold` on another thread; returns once the file is open.
    fn hold(scratch: &Scratch, hold: Duration) -> thread::JoinHandle<()> {
        let folder = scratch.ephemeral().join("Cef").join("1");
        fs::create_dir_all(&folder).expect("held folder");
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .share_mode(0)
            .open(folder.join("held.bin"))
            .expect("open without sharing");
        thread::spawn(move || {
            thread::sleep(hold);
            drop(file);
        })
    }

    #[test]
    fn profile_an_ephemeral_folder_still_in_use_is_emptied_after_retries() {
        let scratch = Scratch::new("held-retry");
        let holder = hold(&scratch, Duration::from_millis(200));

        let lease = acquire(
            &scratch.web(),
            "Cef",
            &scratch.ephemeral(),
            1,
            ProfileKind::Ephemeral,
            Duration::from_secs(2),
        )
        .expect("emptied after the holder let go");
        holder.join().expect("holder thread");

        assert_eq!(lease.kind, ProfileKind::Ephemeral);
        assert_eq!(
            fs::read_dir(&lease.path).expect("read").count(),
            0,
            "the folder is empty"
        );
    }

    #[test]
    fn profile_an_ephemeral_folder_that_stays_in_use_fails_after_the_wait() {
        let scratch = Scratch::new("held-fail");
        let holder = hold(&scratch, Duration::from_secs(1));

        let started = Instant::now();
        let result = acquire(
            &scratch.web(),
            "Cef",
            &scratch.ephemeral(),
            1,
            ProfileKind::Ephemeral,
            Duration::from_millis(200),
        );
        let waited = started.elapsed();
        holder.join().expect("holder thread");

        assert!(result.is_err(), "the folder was still in use");
        assert!(waited >= Duration::from_millis(200), "waited {waited:?}");
    }
}
