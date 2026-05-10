use std::sync::OnceLock;
pub struct OnceLockIntoInnerMapPayload {
    value: String,
}

impl OnceLockIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("once-lock-into-inner-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("once-lock-into-inner-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-once-lock-into-inner-map:{}", self.value)
    }
}

pub fn selected_once_lock_into_inner_map(raw: &str) -> String {
    let slot = OnceLock::new();
    let _ = slot.set(OnceLockIntoInnerMapPayload::new(raw));
    slot.into_inner()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_once_lock_into_inner_map(raw: &str) -> String {
    OnceLockIntoInnerMapPayload::new(raw).unused_label()
}
