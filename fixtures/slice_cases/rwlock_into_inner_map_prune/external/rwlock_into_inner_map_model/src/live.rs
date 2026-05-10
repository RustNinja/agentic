use std::sync::RwLock;
pub struct RwlockIntoInnerMapPayload {
    value: String,
}

impl RwlockIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rwlock-into-inner-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("rwlock-into-inner-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-rwlock-into-inner-map:{}", self.value)
    }
}

pub fn selected_rwlock_into_inner_map(raw: &str) -> String {
    let payload = RwLock::new(RwlockIntoInnerMapPayload::new(raw));
    RwLock::into_inner(payload)
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_rwlock_into_inner_map(raw: &str) -> String {
    RwlockIntoInnerMapPayload::new(raw).unused_label()
}
