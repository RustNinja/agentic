use std::sync::RwLock;

pub struct RwLockWriteMapPayload {
    value: String,
}

impl RwLockWriteMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rwlock-write-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("rwlock-write-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rwlock-write-map:{}", self.value)
    }
}

pub fn selected_rwlock_write_map(raw: &str) -> String {
    let payload = RwLock::new(RwLockWriteMapPayload::new(raw));
    let mut guard = payload.write().expect("fixture rwlock should not poison");
    guard.bump_and_render()
}

pub fn dead_live_rwlock_write_map(raw: &str) -> String {
    RwLockWriteMapPayload::new(raw).dead_method()
}
