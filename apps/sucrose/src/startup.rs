//! Starting with the OS, the service's side (TRAY-03, D-08): what a `SetStartup` request does to Sucrose's own start entry
//! and the settings, and what the Portal is told afterwards.
//!
//! Only Normal and None work. Priority and Scheduler wait for Phase 5 (TRAY-04): asking for them changes nothing.
//! The entry is always the service's own executable, never text from a client (T-03-54), and the C# entries are never
//! touched (T-03-55).

use std::path::Path;

use sucrose_ipc::portal::{StartupInfo, StartupMode};
use sucrose_platform::{AutostartState, PlatformError};

use crate::settings_host::SettingsHost;

/// What the OS side of startup does. The service uses [`SystemStartup`]; the tests use a fake, so the decisions are
/// checked without touching the registry.
pub trait StartupOs {
    /// Writes Sucrose's own entry for `exe`.
    fn enable(&self, exe: &Path) -> Result<(), PlatformError>;
    /// Removes Sucrose's own entry.
    fn disable(&self) -> Result<(), PlatformError>;
    /// What the OS says about Sucrose's own entry for `exe`.
    fn state(&self, exe: &Path) -> AutostartState;
}

/// The start entry of this OS.
pub struct SystemStartup;

impl StartupOs for SystemStartup {
    fn enable(&self, exe: &Path) -> Result<(), PlatformError> {
        sucrose_platform::enable_autostart(exe)
    }

    fn disable(&self) -> Result<(), PlatformError> {
        sucrose_platform::disable_autostart()
    }

    fn state(&self, exe: &Path) -> AutostartState {
        sucrose_platform::autostart_state(exe)
    }
}

/// What a `SetStartup` request came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupChange {
    /// The entry was written and RunStartup is 1.
    Enabled,
    /// The entry was removed and RunStartup is 0.
    Disabled,
    /// Priority and Scheduler are not available yet: nothing changed (D-08).
    Refused,
    /// The entry or the settings could not be written; the previous state stays.
    Failed,
}

/// Whether the entry is where the request wanted it after an OS call that reported an error: such a call can fail
/// half-way, so the state the OS reports decides, not the error.
fn settled_by_state(wanted_present: bool, after: &AutostartState) -> bool {
    after.entry_present == wanted_present
}

/// Applies a request for `mode`: the entry first, then the setting; when the setting cannot be saved the entry goes
/// back to what it was, so the two never disagree.
pub fn set_startup(mode: StartupMode, exe: &Path, os: &dyn StartupOs, host: &mut SettingsHost) -> StartupChange {
    let before = os.state(exe);
    match mode {
        StartupMode::Priority | StartupMode::Scheduler => {
            tracing::info!(?mode, "startup mode is not available yet, nothing changed");
            StartupChange::Refused
        }
        StartupMode::Normal => {
            if let Err(error) = os.enable(exe) {
                if !settled_by_state(true, &os.state(exe)) {
                    tracing::warn!(%error, "the startup entry could not be written");
                    return StartupChange::Failed;
                }
                tracing::warn!(%error, "the OS reported an error but the startup entry is there, so the setting follows it");
            }
            if let Err(reason) = host.update(|settings| settings.general.run_startup = 1) {
                tracing::warn!(%reason, "the startup setting could not be saved");
                // An entry that was there before stays; one this request created goes again.
                if !before.entry_present
                    && let Err(error) = os.disable()
                {
                    tracing::warn!(%error, "the startup entry could not be taken back");
                }
                return StartupChange::Failed;
            }
            tracing::info!("startup entry written");
            StartupChange::Enabled
        }
        StartupMode::None => {
            if let Err(error) = os.disable() {
                if !settled_by_state(false, &os.state(exe)) {
                    tracing::warn!(%error, "the startup entry could not be removed");
                    return StartupChange::Failed;
                }
                tracing::warn!(%error, "the OS reported an error but the startup entry is gone, so the setting follows it");
            }
            if let Err(reason) = host.update(|settings| settings.general.run_startup = 0) {
                tracing::warn!(%reason, "the startup setting could not be saved");
                if before.entry_present
                    && let Err(error) = os.enable(exe)
                {
                    tracing::warn!(%error, "the startup entry could not be taken back");
                }
                return StartupChange::Failed;
            }
            tracing::info!("startup entry removed");
            StartupChange::Disabled
        }
    }
}

/// What the Portal is told: the stored mode with what the OS says about Sucrose's own entry, and `legacy_entry`, the
/// last answer about the C# entry (it is looked up only when the General page opens, never polled).
pub fn startup_info(host: &SettingsHost, exe: &Path, os: &dyn StartupOs, legacy_entry: bool) -> StartupInfo {
    let state = os.state(exe);
    StartupInfo {
        mode: StartupMode::from_run_startup(host.settings().general.run_startup),
        entry_present: state.entry_present,
        disabled_by_os: state.disabled_by_os,
        legacy_entry,
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, fs, path::PathBuf};

    use super::*;

    /// A settings folder of its own, removed when the test ends.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("sucrose-startup-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            Self(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// An OS that remembers what it was asked and can be told to fail.
    #[derive(Default)]
    struct FakeOs {
        present: RefCell<bool>,
        disabled_by_os: bool,
        fail_enable: bool,
        fail_disable: bool,
        /// What the entry looks like after a failed call: a call can fail half-way and still change the OS.
        present_after_failed_enable: Option<bool>,
        present_after_failed_disable: Option<bool>,
        calls: RefCell<Vec<&'static str>>,
    }

    impl StartupOs for FakeOs {
        fn enable(&self, _: &Path) -> Result<(), PlatformError> {
            self.calls.borrow_mut().push("enable");
            if self.fail_enable {
                if let Some(present) = self.present_after_failed_enable {
                    *self.present.borrow_mut() = present;
                }
                return Err(PlatformError::Unsupported("test"));
            }
            *self.present.borrow_mut() = true;
            Ok(())
        }

        fn disable(&self) -> Result<(), PlatformError> {
            self.calls.borrow_mut().push("disable");
            if self.fail_disable {
                if let Some(present) = self.present_after_failed_disable {
                    *self.present.borrow_mut() = present;
                }
                return Err(PlatformError::Unsupported("test"));
            }
            *self.present.borrow_mut() = false;
            Ok(())
        }

        fn state(&self, _: &Path) -> AutostartState {
            AutostartState {
                entry_present: *self.present.borrow(),
                disabled_by_os: self.disabled_by_os,
                legacy_entry: false,
            }
        }
    }

    fn exe() -> &'static Path {
        Path::new("sucrose.exe")
    }

    fn saved_host(scratch: &Scratch, run_startup: u8) -> SettingsHost {
        let (mut host, _) = SettingsHost::open(&scratch.0, None);
        assert!(host.persists(), "the scratch folder can be used");
        host.update(|settings| settings.general.run_startup = run_startup)
            .expect("the value is saved");
        host
    }

    fn stored(host: &SettingsHost) -> u8 {
        host.settings().general.run_startup
    }

    #[test]
    fn normal_writes_the_entry_and_stores_one() {
        let scratch = Scratch::new("normal");
        let mut host = saved_host(&scratch, 0);
        let os = FakeOs::default();

        assert_eq!(
            set_startup(StartupMode::Normal, exe(), &os, &mut host),
            StartupChange::Enabled
        );

        assert_eq!(stored(&host), 1);
        assert_eq!(*os.calls.borrow(), ["enable"]);
        assert!(startup_info(&host, exe(), &os, false).entry_present);
    }

    #[test]
    fn none_removes_the_entry_and_stores_zero() {
        let scratch = Scratch::new("none");
        let mut host = saved_host(&scratch, 1);
        let os = FakeOs::default();
        *os.present.borrow_mut() = true;

        assert_eq!(
            set_startup(StartupMode::None, exe(), &os, &mut host),
            StartupChange::Disabled
        );

        assert_eq!(stored(&host), 0);
        assert_eq!(*os.calls.borrow(), ["disable"]);
        assert!(!startup_info(&host, exe(), &os, false).entry_present);
    }

    #[test]
    fn none_after_a_stored_scheduler_stores_zero_and_clears_the_entry() {
        let scratch = Scratch::new("none-after-scheduler");
        let mut host = saved_host(&scratch, 3);
        let os = FakeOs::default();

        assert_eq!(
            set_startup(StartupMode::None, exe(), &os, &mut host),
            StartupChange::Disabled
        );

        assert_eq!(stored(&host), 0);
    }

    #[test]
    fn priority_and_scheduler_are_refused_and_change_nothing() {
        let scratch = Scratch::new("refused");
        let mut host = saved_host(&scratch, 3);
        let os = FakeOs::default();

        for mode in [StartupMode::Priority, StartupMode::Scheduler] {
            assert_eq!(set_startup(mode, exe(), &os, &mut host), StartupChange::Refused);
        }

        assert_eq!(stored(&host), 3);
        assert!(os.calls.borrow().is_empty(), "no entry is written or removed");
    }

    #[test]
    fn a_failed_entry_write_leaves_the_setting_alone() {
        let scratch = Scratch::new("enable-fails");
        let mut host = saved_host(&scratch, 0);
        let os = FakeOs {
            fail_enable: true,
            ..FakeOs::default()
        };

        assert_eq!(
            set_startup(StartupMode::Normal, exe(), &os, &mut host),
            StartupChange::Failed
        );

        assert_eq!(stored(&host), 0);
    }

    #[test]
    fn a_failed_removal_leaves_the_setting_alone() {
        let scratch = Scratch::new("disable-fails");
        let mut host = saved_host(&scratch, 1);
        let os = FakeOs {
            fail_disable: true,
            ..FakeOs::default()
        };
        *os.present.borrow_mut() = true;

        assert_eq!(
            set_startup(StartupMode::None, exe(), &os, &mut host),
            StartupChange::Failed
        );

        assert_eq!(stored(&host), 1);
    }

    #[test]
    fn a_removal_that_fails_after_the_entry_is_gone_stores_zero() {
        let scratch = Scratch::new("disable-fails-entry-gone");
        let mut host = saved_host(&scratch, 1);
        let os = FakeOs {
            fail_disable: true,
            present_after_failed_disable: Some(false),
            ..FakeOs::default()
        };
        *os.present.borrow_mut() = true;

        assert_eq!(
            set_startup(StartupMode::None, exe(), &os, &mut host),
            StartupChange::Disabled
        );

        assert_eq!(stored(&host), 0, "the setting follows the entry that is gone");
        assert!(!startup_info(&host, exe(), &os, false).entry_present);
    }

    #[test]
    fn a_removal_that_fails_with_the_entry_still_there_leaves_the_setting_alone() {
        let scratch = Scratch::new("disable-fails-entry-there");
        let mut host = saved_host(&scratch, 1);
        let os = FakeOs {
            fail_disable: true,
            present_after_failed_disable: Some(true),
            ..FakeOs::default()
        };
        *os.present.borrow_mut() = true;

        assert_eq!(
            set_startup(StartupMode::None, exe(), &os, &mut host),
            StartupChange::Failed
        );

        assert_eq!(stored(&host), 1);
    }

    #[test]
    fn a_write_that_fails_with_the_entry_present_afterwards_stores_one() {
        let scratch = Scratch::new("enable-fails-entry-there");
        let mut host = saved_host(&scratch, 0);
        let os = FakeOs {
            fail_enable: true,
            present_after_failed_enable: Some(true),
            ..FakeOs::default()
        };

        assert_eq!(
            set_startup(StartupMode::Normal, exe(), &os, &mut host),
            StartupChange::Enabled
        );

        assert_eq!(stored(&host), 1, "the setting follows the entry that is there");
    }

    #[test]
    fn a_setting_that_cannot_be_saved_takes_the_new_entry_back() {
        let mut host = SettingsHost::unsaved(sucrose_config::model::Settings::default());
        let os = FakeOs::default();

        assert_eq!(
            set_startup(StartupMode::Normal, exe(), &os, &mut host),
            StartupChange::Failed
        );

        assert_eq!(*os.calls.borrow(), ["enable", "disable"]);
        assert!(!*os.present.borrow(), "no entry without the setting");
    }

    #[test]
    fn a_setting_that_cannot_be_saved_keeps_an_entry_that_was_there() {
        let mut host = SettingsHost::unsaved(sucrose_config::model::Settings::default());
        let os = FakeOs::default();
        *os.present.borrow_mut() = true;

        assert_eq!(
            set_startup(StartupMode::Normal, exe(), &os, &mut host),
            StartupChange::Failed
        );

        assert!(*os.present.borrow(), "the entry of an earlier Normal stays");
    }

    #[test]
    fn a_removal_whose_setting_cannot_be_saved_puts_the_entry_back() {
        let mut host = SettingsHost::unsaved(sucrose_config::model::Settings::default());
        let os = FakeOs::default();
        *os.present.borrow_mut() = true;

        assert_eq!(
            set_startup(StartupMode::None, exe(), &os, &mut host),
            StartupChange::Failed
        );

        assert_eq!(*os.calls.borrow(), ["disable", "enable"]);
        assert!(*os.present.borrow());
    }

    #[test]
    fn the_info_carries_the_stored_mode_the_os_flags_and_the_legacy_answer() {
        let scratch = Scratch::new("info");
        let host = saved_host(&scratch, 3);
        let os = FakeOs {
            disabled_by_os: true,
            ..FakeOs::default()
        };

        let info = startup_info(&host, exe(), &os, true);

        assert_eq!(
            info,
            StartupInfo {
                mode: StartupMode::Scheduler,
                entry_present: false,
                disabled_by_os: true,
                legacy_entry: true,
            }
        );
    }
}
