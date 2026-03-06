use crate::process::errors::ProcessKillError;
use tracing::{debug, error};

pub struct ProcessHandle {
    /// keeping the field for log/debug purpose.
    // may decide to remove later
    pub pid: u32,
    pub child: tokio::process::Child,
}

impl ProcessHandle {
    /// if child exits on its own child.id() still returns Some(self.pid)
    /// tokio doesn't know the child exited until .wait() is called
    fn check_pid(&mut self) -> Result<bool, ProcessKillError> {
        match self.child.try_wait() {
            Ok(None) => Ok(true),
            Ok(Some(_)) => Ok(false),
            Err(e) => Err(ProcessKillError::WaitFailed(e.to_string())),
        }
    }

    pub async fn graceful_kill(&mut self) -> Result<(), ProcessKillError> {
        debug!(pid = self.pid, "Sending graceful termination signal");

        if !self.check_pid()? {
            debug!(pid = self.pid, "Process already exited");
            return Ok(());
        }

        #[cfg(unix)]
        {
            self.unix_kill(false, true)
        }

        #[cfg(windows)]
        {
            self.windows_kill(false, true).await
        }
    }

    pub async fn force_kill(&mut self) -> Result<(), ProcessKillError> {
        debug!(pid = self.pid, "Sending force kill signal");

        if !self.check_pid()? {
            debug!(pid = self.pid, "Process already exited");
            return Ok(());
        }

        match self.child.kill().await {
            Ok(()) => {
                debug!(pid = self.pid, "Force kill successful");
                Ok(())
            }
            Err(e) => {
                error!(pid = self.pid, error = %e, "Force kill failed, trying platform-specific");
                #[cfg(unix)]
                {
                    self.unix_kill(true, true)
                }
                #[cfg(windows)]
                {
                    self.windows_kill(true, true).await
                }
            }
        }
    }

    /// Wait for process to exit with timeout
    pub async fn wait_timeout(
        &mut self,
        timeout_secs: u64,
    ) -> Result<Option<std::process::ExitStatus>, ProcessKillError> {
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(timeout_secs),
            self.child.wait(),
        )
        .await
        {
            Ok(Ok(status)) => Ok(Some(status)),
            Ok(Err(e)) => Err(ProcessKillError::WaitFailed(e.to_string())),
            Err(_) => Ok(None), // Timeout
        }
    }

    #[cfg(unix)]
    fn unix_kill(&self, force: bool, kill_tree: bool) -> Result<(), ProcessKillError> {
        use nix::sys::signal::{self, Signal};
        use nix::unistd::Pid;

        let kill_signal = if force {
            Signal::SIGKILL
        } else {
            Signal::SIGTERM
        };

        let info_message = format!("for process{}.", if kill_tree { " group" } else { "" });

        let raw = i32::try_from(self.pid).map_err(|_| ProcessKillError::Invalid)?;
        let target = if kill_tree { -raw } else { raw };
        let pid = Pid::from_raw(target);

        signal::kill(pid, kill_signal).map_err(|e| {
            ProcessKillError::SignalFailed(format!(
                "{} failed {}: {}",
                kill_signal.to_string(),
                info_message,
                e
            ))
        })?;

        debug!(
            pid = self.pid,
            "{} sent successfully {}",
            kill_signal.to_string(),
            info_message
        );
        Ok(())
    }

    #[cfg(windows)]
    async fn windows_kill(&self, force: bool, kill_tree: bool) -> Result<(), ProcessKillError> {
        let mut args = Vec::with_capacity(4);

        if force {
            args.push("/F");
        }
        if kill_tree {
            args.push("/T");
        }
        args.push("/PID");

        let pid_str = self.pid.to_string();
        args.push(&pid_str);

        let output = tokio::process::Command::new("taskkill")
            .args(&args)
            .output()
            .await
            .map_err(|e| ProcessKillError::SignalFailed(format!("taskkill failed: {}", e)))?;

        if output.status.success() {
            debug!(
                pid = self.pid,
                "Windows {} kill (taskkill) successful",
                if force { "force" } else { "graceful" }
            );
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ProcessKillError::SignalFailed(format!(
                "taskkill failed: {}",
                stderr
            )))
        }
    }
}
