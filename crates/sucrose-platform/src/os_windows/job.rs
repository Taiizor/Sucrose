//! A job object that ends every renderer when the service goes away, and the spawn that puts a renderer in it.

use std::{
    io, mem,
    os::windows::{io::AsRawHandle, process::CommandExt},
    process::{Child, Command},
    ptr,
};

use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::{
            JobObjects::{
                AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
                JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation, SetInformationJobObject,
                TerminateJobObject,
            },
            Threading::CREATE_NO_WINDOW,
        },
    },
    core::PCWSTR,
};

use crate::PlatformError;

/// The exit code of a process that [`RendererJob::terminate_all`] ends.
const TERMINATED_EXIT_CODE: u32 = 1;

/// A kill-on-close job object. Every renderer the service starts joins it, so the OS ends all of them when the
/// service process ends for any reason, including Task Manager and `taskkill /F` (T-02-23).
#[derive(Debug)]
pub struct RendererJob(HANDLE);

// SAFETY: a job object handle is a kernel object reference that any thread may use; the type never reads through it.
unsafe impl Send for RendererJob {}
// SAFETY: see above; the calls made through `&self` are thread-safe kernel calls.
unsafe impl Sync for RendererJob {}

impl RendererJob {
    /// Creates the job with the kill-on-close limit.
    pub fn new() -> Result<Self, PlatformError> {
        // SAFETY: no security attributes and no name are valid arguments; the result is checked.
        let handle =
            unsafe { CreateJobObjectW(None, PCWSTR::null()) }.map_err(|e| super::os_error("CreateJobObjectW", &e))?;
        // From here on Drop closes the handle if a later step fails.
        let job = Self(handle);

        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: `info` is a fully initialised JOBOBJECT_EXTENDED_LIMIT_INFORMATION of the size passed, matching the
        // JobObjectExtendedLimitInformation class, and it outlives the call.
        unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                ptr::from_ref(&info).cast(),
                u32::try_from(mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>()).unwrap_or(u32::MAX),
            )
        }
        .map_err(|e| super::os_error("SetInformationJobObject", &e))?;
        Ok(job)
    }

    /// Starts `cmd` without a console window and puts the new process in the job. The renderer starts no processes
    /// of its own, so the short time before it joins is harmless. When it cannot join, it is stopped again and the
    /// error is returned: a renderer outside the job could outlive the service.
    pub fn spawn(&self, cmd: &mut Command) -> io::Result<Child> {
        cmd.creation_flags(CREATE_NO_WINDOW.0);
        let mut child = cmd.spawn()?;
        // SAFETY: both handles are valid: the job is owned by `self` and the process handle by `child`, which stays
        // alive for the call.
        let assigned = unsafe { AssignProcessToJobObject(self.0, HANDLE(child.as_raw_handle())) };
        if let Err(error) = assigned {
            // Best effort: the process may already be gone.
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::other(format!("AssignProcessToJobObject failed: {error}")));
        }
        Ok(child)
    }

    /// Ends every process still in the job. Used after the grace period of a shutdown, when only the renderers that
    /// ignored Close are left.
    pub fn terminate_all(&self) -> Result<(), PlatformError> {
        // SAFETY: the job handle is valid for the lifetime of `self`.
        unsafe { TerminateJobObject(self.0, TERMINATED_EXIT_CODE) }
            .map_err(|e| super::os_error("TerminateJobObject", &e))
    }
}

impl Drop for RendererJob {
    fn drop(&mut self) {
        // SAFETY: the handle came from a successful CreateJobObjectW and is closed exactly once, here. Closing it
        // ends the processes in the job (kill on close). A failed close cannot be acted on.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    /// A process that sleeps long enough for the test to end it.
    fn sleeper() -> Command {
        let mut cmd = Command::new("cmd.exe");
        cmd.args(["/C", "ping -n 30 127.0.0.1 >NUL"]);
        cmd
    }

    #[test]
    fn terminate_all_ends_a_spawned_process() {
        let job = RendererJob::new().expect("job");
        let mut child = job.spawn(&mut sleeper()).expect("spawn");

        job.terminate_all().expect("terminate");

        let started = Instant::now();
        loop {
            if child.try_wait().expect("try_wait").is_some() {
                break;
            }
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "the process is still running"
            );
            std::thread::sleep(Duration::from_millis(25));
        }
    }

    #[test]
    fn closing_the_job_ends_a_spawned_process() {
        let job = RendererJob::new().expect("job");
        let mut child = job.spawn(&mut sleeper()).expect("spawn");

        drop(job);

        let started = Instant::now();
        loop {
            if child.try_wait().expect("try_wait").is_some() {
                break;
            }
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "the process outlived its job"
            );
            std::thread::sleep(Duration::from_millis(25));
        }
    }
}
