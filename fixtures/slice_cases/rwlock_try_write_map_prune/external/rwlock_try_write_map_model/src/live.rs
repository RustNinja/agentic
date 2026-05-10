use std::sync::RwLock;
pub struct RwlockTryWriteMapPayload {
    value: String,
}

impl RwlockTryWriteMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rwlock-try-write-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("rwlock-try-write-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-rwlock-try-write-map:{}", self.value)
    }
}

pub fn selected_rwlock_try_write_map(raw: &str) -> String {
    let payload = RwLock::new(RwlockTryWriteMapPayload::new(raw));
    payload
        .try_write()
        .map(|mut payload| payload.bump_and_render())
        .unwrap_or_default()
}

pub fn dead_live_rwlock_try_write_map(raw: &str) -> String {
    RwlockTryWriteMapPayload::new(raw).unused_label()
}
