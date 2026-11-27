//! Installing a missing prerequisite on request (ENG-06, D-15): the Microsoft bootstrapper is downloaded over HTTPS into a
//! private folder, verified by Authenticode (Microsoft Corporation, Microsoft root), run through the shell so Windows
//! shows its own elevation prompt, deleted, and the prerequisite is looked at again. Nothing here runs before the user
//! confirms in the Portal, Sucrose never elevates itself, and a file whose signature does not check out is deleted and
//! never executed (T-04-41, T-04-42, T-04-43). One install runs at a time (T-04-44).
//!
//! The steps that touch the network, the signature check and the installer are behind [`Steps`], so the order of the
//! stages and the cleanup are tested with fakes; [`SystemSteps`] is the real thing.

use std::{
    fs::{self, File},
    io::{Read, Write},
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use sucrose_ipc::portal::{InstallFailureView, InstallStageView, PrerequisiteView};
use sucrose_platform::Arch;

use crate::engines;

/// The most bytes an installer download may have (the Visual C++ bootstrapper is about 25 MiB).
pub const MAX_INSTALLER_BYTES: u64 = 128 << 20;

/// How often the download reports its progress.
const PROGRESS_EVERY: Duration = Duration::from_millis(250);

/// Exit codes of a Microsoft installer that are not a failure: success, "another version is already installed"
/// (ERROR_PRODUCT_VERSION) and "success, restart needed" (ERROR_SUCCESS_REBOOT_REQUIRED). Whether the prerequisite is
/// there now is decided by looking again.
const ACCEPTED_EXIT_CODES: [i32; 3] = [0, 1638, 3010];

/// A part of the machine Sucrose can install for an engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prerequisite {
    /// The Visual C++ Redistributable CEF needs.
    VcRedist,
    /// The WebView2 Runtime (Round B: no download is wired yet).
    WebView2Runtime,
}

impl Prerequisite {
    /// What the Portal asked for.
    pub fn from_view(view: PrerequisiteView) -> Self {
        match view {
            PrerequisiteView::VcRedist => Self::VcRedist,
            PrerequisiteView::WebView2Runtime => Self::WebView2Runtime,
        }
    }

    /// The official Microsoft download of this prerequisite for `arch`, the same short link the C# app uses; `None` where
    /// none is known (WebView2 until Round B, a processor without a Visual C++ runtime).
    pub fn url(self, arch: Arch) -> Option<String> {
        match self {
            Self::VcRedist => {
                let suffix = match arch {
                    Arch::X86 => "x86",
                    Arch::X64 => "x64",
                    Arch::Arm64 => "arm64",
                    Arch::Other => return None,
                };
                Some(format!("https://aka.ms/vc14/vc_redist.{suffix}.exe"))
            }
            Self::WebView2Runtime => None,
        }
    }

    /// The file name the download gets inside its private folder.
    fn file_name(self) -> &'static str {
        match self {
            Self::VcRedist => "vc_redist.exe",
            Self::WebView2Runtime => "webview2_setup.exe",
        }
    }

    /// The installer's command line: no questions, Microsoft's own progress window, no restart.
    fn arguments(self) -> &'static str {
        match self {
            Self::VcRedist => "/install /passive /norestart",
            Self::WebView2Runtime => "",
        }
    }

    /// Whether the machine has this prerequisite now, by the same rule the engine report uses.
    fn installed_now(self) -> bool {
        match self {
            Self::VcRedist => {
                let found = sucrose_platform::current().vc_redist(Arch::current());
                engines::vc_is_fine(found.as_ref())
            }
            Self::WebView2Runtime => false,
        }
    }
}

/// Why an install did not finish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallFailure {
    /// The installer could not be downloaded.
    Download,
    /// The signature check refused the file; it was not run.
    Signature,
    /// The installer did not run to the end, or the prerequisite is still missing afterwards.
    Installer,
}

/// Where an install is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStage {
    /// The installer is being downloaded; `total` is known when the server says so.
    Downloading {
        /// Bytes received so far.
        received: u64,
        /// Bytes expected, when known.
        total: Option<u64>,
    },
    /// The signature is being checked.
    Verifying,
    /// The installer runs (Windows may ask for permission).
    Installing,
    /// The prerequisite is being looked for again.
    Rechecking,
    /// The prerequisite is installed.
    Done,
    /// The install stopped.
    Failed(InstallFailure),
    /// The user stopped the download; nothing was run and nothing is left.
    Cancelled,
}

impl InstallStage {
    /// Whether nothing follows this stage.
    pub fn is_final(self) -> bool {
        matches!(self, Self::Done | Self::Failed(_) | Self::Cancelled)
    }

    /// What the Portal is told.
    pub fn view(self) -> InstallStageView {
        match self {
            Self::Downloading { received, total } => InstallStageView::Downloading { received, total },
            Self::Verifying => InstallStageView::Verifying,
            Self::Installing => InstallStageView::Installing,
            Self::Rechecking => InstallStageView::Rechecking,
            Self::Done => InstallStageView::Done,
            Self::Failed(InstallFailure::Download) => InstallStageView::Failed(InstallFailureView::Download),
            Self::Failed(InstallFailure::Signature) => InstallStageView::Failed(InstallFailureView::Signature),
            Self::Failed(InstallFailure::Installer) => InstallStageView::Failed(InstallFailureView::Installer),
            Self::Cancelled => InstallStageView::Cancelled,
        }
    }
}

/// Why a download ended without a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadError {
    /// The user stopped it.
    Cancelled,
    /// It failed; the text is for the log, never for the user.
    Failed(String),
}

/// The progress callback of a download: bytes received and bytes expected, when known.
pub type Progress<'a> = &'a mut dyn FnMut(u64, Option<u64>);

/// The steps of an install that touch the outside world.
pub trait Steps: Send + Sync {
    /// Downloads `url` to `dest`, calling `progress` now and then and stopping with [`DownloadError::Cancelled`] when `cancel`
    /// is set.
    fn download(
        &self,
        url: &str,
        dest: &Path,
        cancel: &AtomicBool,
        progress: Progress<'_>,
    ) -> Result<(), DownloadError>;

    /// Checks that `path` is signed by Microsoft. `Err` means: do not run it.
    fn verify(&self, path: &Path) -> Result<(), String>;

    /// Runs the installer and returns its exit code.
    fn run(&self, path: &Path, args: &str) -> Result<i32, String>;

    /// Whether the prerequisite is installed now.
    fn installed(&self, prerequisite: Prerequisite) -> bool;
}

/// A private folder under the cache that is removed when it goes out of scope, on every path.
struct PrivateDir(PathBuf);

impl PrivateDir {
    /// Creates a new folder `<root>/Install/<unique>`; it fails when that exists, so no other process's folder is used.
    fn create(root: &Path) -> std::io::Result<Self> {
        let parent = root.join("Install");
        fs::create_dir_all(&parent)?;
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        let dir = parent.join(format!("{}-{nanos}", std::process::id()));
        fs::create_dir(&dir)?;
        Ok(Self(dir))
    }
}

impl Drop for PrivateDir {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            tracing::warn!(%error, "the install folder could not be removed");
        }
    }
}

/// Runs one install to its end, reporting every stage. The folder of the download is under `root` and is gone when this
/// returns. `cancel` stops the download, and also an install that has not started the installer yet.
pub fn run_install(
    prerequisite: Prerequisite,
    arch: Arch,
    steps: &dyn Steps,
    root: &Path,
    cancel: &AtomicBool,
    report: &dyn Fn(InstallStage),
) {
    let Some(url) = prerequisite.url(arch) else {
        tracing::warn!(?prerequisite, ?arch, "no download is known for this prerequisite");
        report(InstallStage::Failed(InstallFailure::Download));
        return;
    };
    report(InstallStage::Downloading {
        received: 0,
        total: None,
    });
    let dir = match PrivateDir::create(root) {
        Ok(dir) => dir,
        Err(error) => {
            tracing::warn!(%error, "the install folder could not be created");
            report(InstallStage::Failed(InstallFailure::Download));
            return;
        }
    };
    let file = dir.0.join(prerequisite.file_name());
    let downloaded = steps.download(&url, &file, cancel, &mut |received, total| {
        report(InstallStage::Downloading { received, total });
    });
    match downloaded {
        Ok(()) => {}
        Err(DownloadError::Cancelled) => {
            tracing::info!("the installer download was stopped");
            report(InstallStage::Cancelled);
            return;
        }
        Err(DownloadError::Failed(reason)) => {
            tracing::warn!(%reason, "the installer could not be downloaded");
            report(InstallStage::Failed(InstallFailure::Download));
            return;
        }
    }
    report(InstallStage::Verifying);
    if let Err(reason) = steps.verify(&file) {
        tracing::warn!(%reason, "the installer is not signed by Microsoft, so it was deleted and not run");
        report(InstallStage::Failed(InstallFailure::Signature));
        return;
    }
    if cancel.load(Ordering::SeqCst) {
        tracing::info!("the install was stopped before the installer started");
        report(InstallStage::Cancelled);
        return;
    }
    report(InstallStage::Installing);
    let outcome = steps.run(&file, prerequisite.arguments());
    drop(dir);
    match outcome {
        Ok(code) if ACCEPTED_EXIT_CODES.contains(&code) => {}
        Ok(code) => {
            tracing::warn!(code, "the installer ended with an error code");
            report(InstallStage::Failed(InstallFailure::Installer));
            return;
        }
        Err(reason) => {
            tracing::warn!(%reason, "the installer did not run");
            report(InstallStage::Failed(InstallFailure::Installer));
            return;
        }
    }
    report(InstallStage::Rechecking);
    if steps.installed(prerequisite) {
        tracing::info!(?prerequisite, "the prerequisite is installed");
        report(InstallStage::Done);
    } else {
        tracing::warn!(
            ?prerequisite,
            "the installer ended but the prerequisite is still missing"
        );
        report(InstallStage::Failed(InstallFailure::Installer));
    }
}

/// The install thread of one request.
pub struct InstallWorker {
    cancel: Arc<AtomicBool>,
}

impl InstallWorker {
    /// Starts the `install` thread; every stage goes to `report`, also when the thread panics (then as a failed installer).
    /// `Err` when the thread cannot start.
    pub fn start(
        prerequisite: Prerequisite,
        steps: Arc<dyn Steps>,
        root: PathBuf,
        report: impl Fn(InstallStage) + Send + 'static,
    ) -> std::io::Result<Self> {
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancel);
        thread::Builder::new().name("install".to_owned()).spawn(move || {
            let ran = catch_unwind(AssertUnwindSafe(|| {
                run_install(prerequisite, Arch::current(), &*steps, &root, &flag, &report);
            }));
            if ran.is_err() {
                tracing::error!("the install thread panicked");
                report(InstallStage::Failed(InstallFailure::Installer));
            }
        })?;
        Ok(Self { cancel })
    }

    /// Stops the download (or an install that has not started its installer); the thread reports `Cancelled`.
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }
}

/// The real steps: ureq over HTTPS, the platform's signature check and shell launch.
pub struct SystemSteps;

impl Steps for SystemSteps {
    fn download(
        &self,
        url: &str,
        dest: &Path,
        cancel: &AtomicBool,
        progress: Progress<'_>,
    ) -> Result<(), DownloadError> {
        let failed = |what: &str, error: &dyn std::fmt::Display| DownloadError::Failed(format!("{what}: {error}"));
        if !url.starts_with("https://") {
            return Err(DownloadError::Failed("the address is not https".to_owned()));
        }
        // https_only also refuses a redirect to plain http; the signature check, not the address, is the authority.
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .https_only(true)
            .timeout_connect(Some(Duration::from_secs(15)))
            .timeout_per_read(Some(Duration::from_secs(30)))
            .timeout_global(Some(Duration::from_secs(30 * 60)))
            .build()
            .into();
        let response = agent
            .get(url)
            .call()
            .map_err(|error| failed("the request failed", &error))?;
        let total = response.body().content_length();
        if total.is_some_and(|total| total > MAX_INSTALLER_BYTES) {
            return Err(DownloadError::Failed("the installer is larger than allowed".to_owned()));
        }
        let mut body = response.into_body();
        let mut reader = body.with_config().limit(MAX_INSTALLER_BYTES).reader();
        let mut out = File::create_new(dest).map_err(|error| failed("the file could not be created", &error))?;
        let mut buffer = vec![0_u8; 64 * 1024];
        let mut received = 0_u64;
        let mut last_report = Instant::now();
        loop {
            if cancel.load(Ordering::SeqCst) {
                return Err(DownloadError::Cancelled);
            }
            let count = reader
                .read(&mut buffer)
                .map_err(|error| failed("the download broke off", &error))?;
            if count == 0 {
                break;
            }
            out.write_all(&buffer[..count])
                .map_err(|error| failed("the file could not be written", &error))?;
            received += count as u64;
            if last_report.elapsed() >= PROGRESS_EVERY {
                last_report = Instant::now();
                progress(received, total);
            }
        }
        out.flush()
            .map_err(|error| failed("the file could not be written", &error))?;
        if total.is_some_and(|total| total != received) {
            return Err(DownloadError::Failed(format!("{received} of {total:?} bytes arrived")));
        }
        progress(received, total);
        Ok(())
    }

    fn verify(&self, path: &Path) -> Result<(), String> {
        sucrose_platform::current()
            .verify_microsoft_signature(path)
            .map(|signer| tracing::info!(signer = signer.subject, "the installer is signed by Microsoft"))
            .map_err(|error| error.to_string())
    }

    fn run(&self, path: &Path, args: &str) -> Result<i32, String> {
        sucrose_platform::current()
            .run_installer(path, args)
            .map_err(|error| error.to_string())
    }

    fn installed(&self, prerequisite: Prerequisite) -> bool {
        prerequisite.installed_now()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    /// A scratch folder of its own, removed when the test ends.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("sucrose-install-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("the scratch folder can be created");
            Self(dir)
        }

        fn install_folder_is_empty(&self) -> bool {
            fs::read_dir(self.0.join("Install")).is_ok_and(|mut entries| entries.next().is_none())
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// What each fake step does, and what it saw.
    struct Fake {
        download: Result<(), DownloadError>,
        verify: Result<(), String>,
        run: Result<i32, String>,
        installed: bool,
        calls: Mutex<Vec<&'static str>>,
        run_args: Mutex<Vec<String>>,
        file_seen_at_run: Mutex<Option<bool>>,
    }

    impl Fake {
        fn working() -> Self {
            Self {
                download: Ok(()),
                verify: Ok(()),
                run: Ok(0),
                installed: true,
                calls: Mutex::new(Vec::new()),
                run_args: Mutex::new(Vec::new()),
                file_seen_at_run: Mutex::new(None),
            }
        }

        fn run_args(&self) -> Vec<String> {
            self.run_args.lock().expect("run args").clone()
        }

        fn calls(&self) -> Vec<&'static str> {
            self.calls.lock().expect("calls").clone()
        }
    }

    impl Steps for Fake {
        fn download(
            &self,
            url: &str,
            dest: &Path,
            cancel: &AtomicBool,
            progress: Progress<'_>,
        ) -> Result<(), DownloadError> {
            self.calls.lock().expect("calls").push("download");
            assert!(url.starts_with("https://"));
            progress(5, Some(10));
            if cancel.load(Ordering::SeqCst) {
                return Err(DownloadError::Cancelled);
            }
            fs::write(dest, b"installer").expect("the fake download writes");
            self.download.clone()
        }

        fn verify(&self, path: &Path) -> Result<(), String> {
            self.calls.lock().expect("calls").push("verify");
            assert!(path.is_file());
            self.verify.clone()
        }

        fn run(&self, path: &Path, args: &str) -> Result<i32, String> {
            self.calls.lock().expect("calls").push("run");
            self.run_args.lock().expect("run args").push(args.to_owned());
            *self.file_seen_at_run.lock().expect("seen") = Some(path.is_file());
            self.run.clone()
        }

        fn installed(&self, _prerequisite: Prerequisite) -> bool {
            self.calls.lock().expect("calls").push("installed");
            self.installed
        }
    }

    /// Runs an install with `fake` and returns the stages it reported.
    fn stages(fake: &Fake, scratch: &Scratch, cancel: &AtomicBool) -> Vec<InstallStage> {
        let seen = Mutex::new(Vec::new());
        run_install(Prerequisite::VcRedist, Arch::X64, fake, &scratch.0, cancel, &|stage| {
            seen.lock().expect("stages").push(stage);
        });
        seen.into_inner().expect("stages")
    }

    #[test]
    fn a_good_install_goes_through_every_stage_and_leaves_nothing() {
        let scratch = Scratch::new("good");
        let fake = Fake::working();

        let seen = stages(&fake, &scratch, &AtomicBool::new(false));

        assert_eq!(
            seen,
            vec![
                InstallStage::Downloading {
                    received: 0,
                    total: None
                },
                InstallStage::Downloading {
                    received: 5,
                    total: Some(10)
                },
                InstallStage::Verifying,
                InstallStage::Installing,
                InstallStage::Rechecking,
                InstallStage::Done,
            ]
        );
        assert_eq!(fake.calls(), ["download", "verify", "run", "installed"]);
        assert_eq!(fake.run_args(), ["/install /passive /norestart"]);
        assert_eq!(*fake.file_seen_at_run.lock().expect("seen"), Some(true));
        assert!(scratch.install_folder_is_empty());
    }

    #[test]
    fn a_refused_signature_deletes_the_file_and_never_runs_it() {
        let scratch = Scratch::new("signature");
        let fake = Fake {
            verify: Err("not signed".to_owned()),
            ..Fake::working()
        };

        let seen = stages(&fake, &scratch, &AtomicBool::new(false));

        assert_eq!(seen.last(), Some(&InstallStage::Failed(InstallFailure::Signature)));
        assert_eq!(fake.calls(), ["download", "verify"]);
        assert!(scratch.install_folder_is_empty());
    }

    #[test]
    fn a_failed_download_is_a_download_failure() {
        let scratch = Scratch::new("download");
        let fake = Fake {
            download: Err(DownloadError::Failed("offline".to_owned())),
            ..Fake::working()
        };

        let seen = stages(&fake, &scratch, &AtomicBool::new(false));

        assert_eq!(seen.last(), Some(&InstallStage::Failed(InstallFailure::Download)));
        assert_eq!(fake.calls(), ["download"]);
        assert!(scratch.install_folder_is_empty());
    }

    #[test]
    fn an_installer_error_code_or_launch_failure_is_an_installer_failure() {
        for run in [Ok(1603), Err("refused".to_owned())] {
            let scratch = Scratch::new("installer");
            let fake = Fake { run, ..Fake::working() };

            let seen = stages(&fake, &scratch, &AtomicBool::new(false));

            assert_eq!(seen.last(), Some(&InstallStage::Failed(InstallFailure::Installer)));
            assert_eq!(fake.calls(), ["download", "verify", "run"]);
            assert!(scratch.install_folder_is_empty());
        }
    }

    #[test]
    fn restart_and_already_installed_codes_count_when_the_prerequisite_is_there() {
        for code in [3010, 1638] {
            let scratch = Scratch::new("codes");
            let fake = Fake {
                run: Ok(code),
                ..Fake::working()
            };

            assert_eq!(
                stages(&fake, &scratch, &AtomicBool::new(false)).last(),
                Some(&InstallStage::Done)
            );
        }
    }

    #[test]
    fn a_prerequisite_that_is_still_missing_afterwards_is_an_installer_failure() {
        let scratch = Scratch::new("still-missing");
        let fake = Fake {
            installed: false,
            ..Fake::working()
        };

        let seen = stages(&fake, &scratch, &AtomicBool::new(false));

        assert_eq!(seen.last(), Some(&InstallStage::Failed(InstallFailure::Installer)));
    }

    #[test]
    fn cancelling_during_the_download_ends_cancelled_with_no_file_left() {
        let scratch = Scratch::new("cancel");
        let fake = Fake::working();

        let seen = stages(&fake, &scratch, &AtomicBool::new(true));

        assert_eq!(seen.last(), Some(&InstallStage::Cancelled));
        assert_eq!(fake.calls(), ["download"]);
        assert!(scratch.install_folder_is_empty());
    }

    #[test]
    fn a_stop_that_arrives_after_the_download_still_keeps_the_installer_from_running() {
        struct StopAfterDownload(Fake);
        impl Steps for StopAfterDownload {
            fn download(
                &self,
                url: &str,
                dest: &Path,
                cancel: &AtomicBool,
                progress: Progress<'_>,
            ) -> Result<(), DownloadError> {
                let done = self.0.download(url, dest, &AtomicBool::new(false), progress);
                cancel.store(true, Ordering::SeqCst);
                done
            }
            fn verify(&self, path: &Path) -> Result<(), String> {
                self.0.verify(path)
            }
            fn run(&self, path: &Path, args: &str) -> Result<i32, String> {
                self.0.run(path, args)
            }
            fn installed(&self, prerequisite: Prerequisite) -> bool {
                self.0.installed(prerequisite)
            }
        }
        let scratch = Scratch::new("late-stop");
        let steps = StopAfterDownload(Fake::working());
        let seen = Mutex::new(Vec::new());

        run_install(
            Prerequisite::VcRedist,
            Arch::X64,
            &steps,
            &scratch.0,
            &AtomicBool::new(false),
            &|stage| seen.lock().expect("stages").push(stage),
        );

        assert_eq!(
            seen.into_inner().expect("stages").last(),
            Some(&InstallStage::Cancelled)
        );
        assert_eq!(steps.0.calls(), ["download", "verify"]);
        assert!(scratch.install_folder_is_empty());
    }

    #[test]
    fn the_download_address_follows_the_architecture() {
        let url = |arch| Prerequisite::VcRedist.url(arch);
        assert_eq!(url(Arch::X64).as_deref(), Some("https://aka.ms/vc14/vc_redist.x64.exe"));
        assert_eq!(url(Arch::X86).as_deref(), Some("https://aka.ms/vc14/vc_redist.x86.exe"));
        assert_eq!(
            url(Arch::Arm64).as_deref(),
            Some("https://aka.ms/vc14/vc_redist.arm64.exe")
        );
        assert_eq!(url(Arch::Other), None);
    }

    #[test]
    fn the_webview2_bootstrapper_has_one_https_address_for_every_architecture_and_silent_arguments() {
        for arch in [Arch::X86, Arch::X64, Arch::Arm64] {
            assert_eq!(
                Prerequisite::WebView2Runtime.url(arch).as_deref(),
                Some("https://go.microsoft.com/fwlink/p/?LinkId=2124703"),
                "{arch:?}"
            );
        }
        assert_eq!(Prerequisite::WebView2Runtime.url(Arch::Other), None);
        assert_eq!(Prerequisite::WebView2Runtime.arguments(), "/silent /install");
        assert_eq!(Prerequisite::WebView2Runtime.file_name(), "webview2_setup.exe");
    }

    #[test]
    fn a_webview2_install_runs_the_bootstrapper_silently_and_is_done_when_the_runtime_is_there() {
        let scratch = Scratch::new("webview2");
        let fake = Fake::working();
        let seen = Mutex::new(Vec::new());

        run_install(
            Prerequisite::WebView2Runtime,
            Arch::X64,
            &fake,
            &scratch.0,
            &AtomicBool::new(false),
            &|stage| seen.lock().expect("stages").push(stage),
        );

        assert_eq!(seen.into_inner().expect("stages").last(), Some(&InstallStage::Done));
        assert_eq!(fake.calls(), ["download", "verify", "run", "installed"]);
        assert_eq!(fake.run_args(), ["/silent /install"]);
        assert!(scratch.install_folder_is_empty());
    }

    #[test]
    fn a_webview2_install_whose_runtime_is_still_missing_afterwards_is_an_installer_failure() {
        let scratch = Scratch::new("webview2-missing");
        let fake = Fake {
            installed: false,
            ..Fake::working()
        };
        let seen = Mutex::new(Vec::new());

        run_install(
            Prerequisite::WebView2Runtime,
            Arch::Arm64,
            &fake,
            &scratch.0,
            &AtomicBool::new(false),
            &|stage| seen.lock().expect("stages").push(stage),
        );

        assert_eq!(
            seen.into_inner().expect("stages").last(),
            Some(&InstallStage::Failed(InstallFailure::Installer))
        );
        assert!(scratch.install_folder_is_empty());
    }

    #[test]
    fn a_prerequisite_without_a_download_fails_before_anything_is_created() {
        let scratch = Scratch::new("no-url");
        let seen = Mutex::new(Vec::new());

        run_install(
            Prerequisite::WebView2Runtime,
            Arch::Other,
            &Fake::working(),
            &scratch.0,
            &AtomicBool::new(false),
            &|stage| seen.lock().expect("stages").push(stage),
        );

        assert_eq!(
            seen.into_inner().expect("stages"),
            vec![InstallStage::Failed(InstallFailure::Download)]
        );
        assert!(!scratch.0.join("Install").exists());
    }

    #[test]
    fn stages_map_to_the_portal_view_and_only_the_last_ones_are_final() {
        assert_eq!(
            InstallStage::Failed(InstallFailure::Signature).view(),
            InstallStageView::Failed(InstallFailureView::Signature)
        );
        assert_eq!(InstallStage::Cancelled.view(), InstallStageView::Cancelled);
        assert!(InstallStage::Done.is_final());
        assert!(InstallStage::Cancelled.is_final());
        assert!(InstallStage::Failed(InstallFailure::Download).is_final());
        assert!(!InstallStage::Verifying.is_final());
        assert!(
            !InstallStage::Downloading {
                received: 1,
                total: None
            }
            .is_final()
        );
    }

    /// A folder that is removed when it goes out of scope and, unless the test is already failing, must be gone then.
    struct MustBeGone(PathBuf);

    impl Drop for MustBeGone {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
            if !thread::panicking() {
                assert!(!self.0.exists(), "{} was not removed", self.0.display());
            }
        }
    }

    /// The system's download and signature check; the run only records what it was asked to run, and a copy of the verified
    /// file is kept in `keep` for the tampering check.
    struct RecordedRun {
        keep: PathBuf,
        signer: Mutex<Option<String>>,
        ran: Mutex<Vec<String>>,
    }

    impl Steps for RecordedRun {
        fn download(
            &self,
            url: &str,
            dest: &Path,
            cancel: &AtomicBool,
            progress: Progress<'_>,
        ) -> Result<(), DownloadError> {
            SystemSteps.download(url, dest, cancel, progress)
        }

        fn verify(&self, path: &Path) -> Result<(), String> {
            let signer = sucrose_platform::current()
                .verify_microsoft_signature(path)
                .map_err(|error| error.to_string())?;
            *self.signer.lock().expect("signer") = Some(signer.subject);
            fs::copy(path, &self.keep).map_err(|error| error.to_string())?;
            Ok(())
        }

        fn run(&self, _path: &Path, args: &str) -> Result<i32, String> {
            self.ran.lock().expect("ran").push(args.to_owned());
            Ok(0)
        }

        fn installed(&self, _prerequisite: Prerequisite) -> bool {
            true
        }
    }

    /// The real Evergreen Bootstrapper goes through the worker up to the run step (ENG-06, D-15): downloaded over HTTPS
    /// from the fwlink address, accepted by the Microsoft signature check, handed to the run step with the silent
    /// arguments (which only records them), and a copy with one byte changed is refused. Needs the network.
    #[test]
    #[ignore = "downloads the real WebView2 Bootstrapper from Microsoft"]
    #[allow(
        clippy::print_stdout,
        reason = "the signer the check reports is quoted in the plan summary"
    )]
    fn webview2_bootstrapper_downloads_and_passes_the_signature_check() {
        let root = MustBeGone(std::env::temp_dir().join(format!("sucrose-webview2-download-{}", std::process::id())));
        let tamper = MustBeGone(std::env::temp_dir().join(format!("sucrose-webview2-tamper-{}", std::process::id())));
        let _ = fs::remove_dir_all(&root.0);
        let _ = fs::remove_dir_all(&tamper.0);
        let copy_dir = PrivateDir::create(&tamper.0).expect("the second private folder can be created");
        let steps = RecordedRun {
            keep: copy_dir.0.join("webview2_setup.exe"),
            signer: Mutex::new(None),
            ran: Mutex::new(Vec::new()),
        };
        let seen = Mutex::new(Vec::new());

        run_install(
            Prerequisite::WebView2Runtime,
            Arch::current(),
            &steps,
            &root.0,
            &AtomicBool::new(false),
            &|stage| seen.lock().expect("stages").push(stage),
        );

        let seen = seen.into_inner().expect("stages");
        assert!(seen.contains(&InstallStage::Verifying), "{seen:?}");
        assert_eq!(seen.last(), Some(&InstallStage::Done), "{seen:?}");
        assert_eq!(*steps.ran.lock().expect("ran"), ["/silent /install"]);
        let signer = steps.signer.lock().expect("signer").clone();
        println!("signer reported by the check: {signer:?}");
        assert_eq!(signer.as_deref(), Some("Microsoft Corporation"));
        assert!(
            fs::read_dir(root.0.join("Install")).is_ok_and(|mut entries| entries.next().is_none()),
            "the download folder is gone"
        );

        let mut bytes = fs::read(&steps.keep).expect("the kept copy can be read");
        let middle = bytes.len() / 2;
        bytes[middle] ^= 0xFF;
        fs::write(&steps.keep, &bytes).expect("the changed copy can be written");
        let refused = SystemSteps.verify(&steps.keep);
        println!("one byte changed: {refused:?}");
        assert!(refused.is_err(), "a copy with one byte changed must be refused");
    }
}
