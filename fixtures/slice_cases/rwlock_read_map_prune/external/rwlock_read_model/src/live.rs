use std::sync::RwLock;

pub struct RwLockReadMapPayload {
    value: String,
}

impl RwLockReadMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rwlock-read-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("rwlock-read-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rwlock-read-map:{}", self.value)
    }
}

pub fn selected_rwlock_read_map(raw: &str) -> String {
    let payload = RwLock::new(RwLockReadMapPayload::new(raw));
    let guard = payload.read().expect("fixture rwlock should not poison");
    guard.render_label()
}

pub fn dead_live_rwlock_read_map(raw: &str) -> String {
    RwLockReadMapPayload::new(raw).dead_method()
}
