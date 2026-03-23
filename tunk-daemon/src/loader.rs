use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use tokio::fs::{self, read_to_string};
use tunk_common::{
    daemon_config::{BackendConfig, TunnelConfig},
    error::DaemonError,
};

async fn list_files_only(dir_path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let mut entries = fs::read_dir(dir_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        // file_type() тоже асинхронный в tokio
        if entry.file_type().await?.is_file() {
            files.push(entry.path());
        }
    }

    Ok(files)
}

async fn parse_backend_data(data: String) -> Result<BackendConfig, ron::Error> {
    let deserialized: BackendConfig = ron::from_str(&data)?;

    Ok(deserialized)
}

async fn parse_tunnel_data(data: String) -> Result<TunnelConfig, ron::Error> {
    let deserialized: TunnelConfig = ron::from_str(&data)?;

    Ok(deserialized)
}

fn get_file_stem(path: &Path) -> Option<String> {
    path.file_stem().and_then(|s| s.to_str()).map(String::from)
}

pub(crate) async fn load_backends(
    backend_path: &Path,
) -> Result<HashMap<String, BackendConfig>, DaemonError> {
    let mut result: HashMap<String, BackendConfig> = HashMap::new();

    let backend_files = list_files_only(backend_path).await?;

    for file_path in backend_files {
        if let Some(file_stem) = get_file_stem(&file_path) {
            let backend_data = read_to_string(file_path).await?;

            match parse_backend_data(backend_data).await {
                Ok(config) => {
                    result.insert(file_stem, config);
                }
                Err(e) => {
                    return Err(DaemonError::InvalidConfig(
                        format!("Error when loading backend config {}: {}", file_stem, e)
                            .to_owned(),
                    ));
                }
            };
        }
    }

    Ok(result)
}

pub(crate) async fn load_tunnels(
    tunnel_path: &Path,
) -> Result<HashMap<String, TunnelConfig>, DaemonError> {
    let mut result: HashMap<String, TunnelConfig> = HashMap::new();

    let tunnel_files = list_files_only(tunnel_path).await?;

    for file_path in tunnel_files {
        if let Some(file_stem) = get_file_stem(&file_path) {
            let tunnel_data = read_to_string(file_path).await?;

            match parse_tunnel_data(tunnel_data).await {
                Ok(config) => {
                    result.insert(file_stem, config);
                }
                Err(e) => {
                    return Err(DaemonError::InvalidConfig(
                        format!("Error when loading tunnel config {}: {}", file_stem, e).to_owned(),
                    ));
                }
            };
        }
    }

    Ok(result)
}
