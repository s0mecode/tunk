use std::{process::Stdio, sync::Arc};

use async_trait::async_trait;
use tokio::{
    process::{Child, Command},
    sync::Mutex,
};
use tunk_common::{
    backend_options::BackendController,
    daemon_config::{BackendConfig, TunnelConfig, TunnelDirection},
    error::{BackendError, ProcessError},
};

struct SecureShellInner {
    child: Option<Child>,
}

pub struct SecureShellConnection {
    inner: Arc<Mutex<SecureShellInner>>,
}

impl SecureShellConnection {
    fn build_ssh_command(config: &TunnelConfig) -> Result<(String, Vec<String>), BackendError> {
        let (remote, auth) = match &config.backend {
            BackendConfig::SecureShell(remote, auth) => (remote, auth),
            _ => {
                return Err(BackendError::InvalidConfig(
                    "Auth config not found or not Secure Shell.".to_owned(),
                ));
            }
        };

        let ssh_bin = std::env::var("SSH_BINARY").unwrap_or_else(|_| "ssh".to_string());
        let mut args = vec![
            "-o".to_string(),
            "StrictHostKeyChecking=no".to_string(),
            "-o".to_string(),
            "UserKnownHostsFile=/dev/null".to_string(),
        ];

        if let Some(key_path) = &auth.key_path {
            args.push("-i".to_string());
            args.push(key_path.to_string_lossy().to_string());
        }

        let tunnel_spec = match config.direction {
            TunnelDirection::Forward => {
                format!(
                    "-L{}:{}:{}:{}",
                    config.local.ip, config.local.port, config.remote.ip, config.remote.port
                )
            }
            TunnelDirection::Reverse => {
                format!(
                    "-R{}:{}:{}:{}",
                    config.remote.ip, config.remote.port, config.local.ip, config.local.port
                )
            }
        };
        args.push(tunnel_spec);

        let target = format!("{}@{}", auth.username, remote.ip);
        args.push(target);
        args.push("-N".to_string());

        Ok((ssh_bin, args))
    }

    pub async fn connect(config: TunnelConfig) -> Result<Self, BackendError> {
        let (cmd, args) = Self::build_ssh_command(&config)?;

        let child = Command::new(&cmd)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                BackendError::ProcessError(ProcessError::ProcessSpawnFailed(e.to_string()))
            })?;

        Ok(Self {
            inner: Arc::new(Mutex::new(SecureShellInner { child: Some(child) })),
        })
    }
}

#[async_trait]
impl BackendController for SecureShellConnection {
    async fn disconnect(&mut self) -> Result<(), BackendError> {
        let mut inner = self.inner.lock().await;

        if let Some(mut child) = inner.child.take() {
            child.kill().await.map_err(|e| {
                BackendError::ProcessError(ProcessError::ProcessKillFailed(e.to_string()))
            })?;

            child.wait().await.map_err(|e| {
                BackendError::ProcessError(ProcessError::ProcessWaitFailed(e.to_string()))
            })?;
        }

        Ok(())
    }

    async fn is_alive(&self) -> bool {
        let mut inner = self.inner.lock().await;
        if let Some(ref mut child) = inner.child {
            match child.try_wait().ok() {
                Some(Some(_)) => return false,
                _ => return true,
            }
        }
        false
    }
}

impl Drop for SecureShellConnection {
    fn drop(&mut self) {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            let mut guard = inner.lock().await;
            if let Some(mut child) = guard.child.take() {
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
        });
    }
}
