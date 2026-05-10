use std::{borrow::Cow, path::PathBuf, sync::Arc, time::Duration};

pub struct ReconnectState {
    id: String,
    socket_path: Option<Arc<PathBuf>>,
    retry_after: Duration,
    fallback_label: Cow<'static, str>,
}

impl ReconnectState {
    pub fn new(raw: &str) -> Self {
        Self {
            id: normalize_reconnect_id(raw),
            socket_path: None,
            retry_after: Duration::from_secs(5),
            fallback_label: Cow::Borrowed("fallback"),
        }
    }

    pub fn summary(&self) -> String {
        format!("reconnect:{}", self.id)
    }

    pub fn dead_details(&self) -> String {
        format!(
            "dead:{}:{}:{}",
            self.socket_path.is_some(),
            self.retry_after.as_secs(),
            self.fallback_label
        )
    }
}

pub fn build_reconnect(raw: &str) -> ReconnectState {
    ReconnectState::new(raw)
}

fn normalize_reconnect_id(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

pub fn dead_live_reconnect(raw: &str) -> String {
    ReconnectState::new(raw).dead_details()
}
