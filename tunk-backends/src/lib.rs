use tunk_common::backend_options::BackendController;
use tunk_common::error::BackendError;

use tunk_common::daemon_config::{BackendConfig, TunnelConfig};

pub mod secure_shell;

/// Factory function to create backend from config
pub async fn create_backend(
    config: TunnelConfig,
) -> Result<Box<dyn BackendController>, BackendError> {
    match config.backend {
        BackendConfig::SecureShell(_, _) => {
            let backend = secure_shell::SecureShellConnection::connect(config).await?;
            Ok(Box::new(backend))
        }
        BackendConfig::File(_) => Err(BackendError::InvalidConfig(
            "Backend should not be a File at this point.".to_string(),
        )),
    }
}
