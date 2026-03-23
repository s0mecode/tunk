use std::sync::Arc;

use tunk_common::daemon_state::DaemonState;

pub async fn list_tunnels(state: Arc<DaemonState>) -> Vec<String> {
    state.tunnels.iter().map(|r| r.key().clone()).collect()
}
