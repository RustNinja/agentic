use std::sync::RwLock;
pub struct RwlockGetMutMapPayload {
    value: String,
}

impl RwlockGetMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rwlock-get-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("rwlock-get-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-rwlock-get-mut-map:{}", self.value)
    }
}

pub fn selected_rwlock_get_mut_map(raw: &str) -> String {
    let mut payload = RwLock::new(RwlockGetMutMapPayload::new(raw));
    payload
        .get_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_default()
}

pub fn dead_live_rwlock_get_mut_map(raw: &str) -> String {
    RwlockGetMutMapPayload::new(raw).unused_label()
}
