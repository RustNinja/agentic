use std::sync::RwLock;
pub struct RwlockTryReadMapPayload {
    value: String,
}

impl RwlockTryReadMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rwlock-try-read-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("rwlock-try-read-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-rwlock-try-read-map:{}", self.value)
    }
}

pub fn selected_rwlock_try_read_map(raw: &str) -> String {
    let payload = RwLock::new(RwlockTryReadMapPayload::new(raw));
    payload
        .try_read()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_rwlock_try_read_map(raw: &str) -> String {
    RwlockTryReadMapPayload::new(raw).unused_label()
}
