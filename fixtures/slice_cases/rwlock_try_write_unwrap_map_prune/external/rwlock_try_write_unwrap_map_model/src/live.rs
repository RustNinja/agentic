use std::sync::RwLock;
#[derive(Debug)]
pub struct RwlockTryWriteUnwrapMapPayload {
    value: String,
}

impl RwlockTryWriteUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rwlock-try-write-unwrap-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("rwlock-try-write-unwrap-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-rwlock-try-write-unwrap-map:{}", self.value)
    }
}

pub fn selected_rwlock_try_write_unwrap_map(raw: &str) -> String {
    let payload = RwLock::new(RwlockTryWriteUnwrapMapPayload::new(raw));
    let label = payload.try_write().unwrap().bump_and_render();
    label
}

pub fn dead_live_rwlock_try_write_unwrap_map(raw: &str) -> String {
    RwlockTryWriteUnwrapMapPayload::new(raw).unused_label()
}
