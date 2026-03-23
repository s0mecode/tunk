use async_trait::async_trait;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::BackendError;

#[async_trait]
pub trait BackendController: Send + Sync {
    async fn disconnect(&mut self) -> Result<(), BackendError>;
    async fn is_alive(&self) -> bool {
        // Default implementation
        false
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SecureShellAuth {
    pub username: String,
    pub key_path: Option<Box<Path>>,
    pub password: Option<String>,
}
