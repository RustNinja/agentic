use std::sync::RwLock;
#[derive(Debug)]
pub struct RwlockTryReadUnwrapMapPayload {
    value: String,
}

impl RwlockTryReadUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rwlock-try-read-unwrap-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("rwlock-try-read-unwrap-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-rwlock-try-read-unwrap-map:{}", self.value)
    }
}

pub fn selected_rwlock_try_read_unwrap_map(raw: &str) -> String {
    let payload = RwLock::new(RwlockTryReadUnwrapMapPayload::new(raw));
    let label = payload.try_read().unwrap().render_label();
    label
}

pub fn dead_live_rwlock_try_read_unwrap_map(raw: &str) -> String {
    RwlockTryReadUnwrapMapPayload::new(raw).unused_label()
}
