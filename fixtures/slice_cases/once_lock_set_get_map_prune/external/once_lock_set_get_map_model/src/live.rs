use std::sync::OnceLock;
pub struct OnceLockSetGetMapPayload {
    value: String,
}

impl OnceLockSetGetMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("once-lock-set-get-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("once-lock-set-get-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-once-lock-set-get-map:{}", self.value)
    }
}

pub fn selected_once_lock_set_get_map(raw: &str) -> String {
    let slot = OnceLock::new();
    let _ = slot.set(OnceLockSetGetMapPayload::new(raw));
    slot.get()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_once_lock_set_get_map(raw: &str) -> String {
    OnceLockSetGetMapPayload::new(raw).unused_label()
}
