use std::sync::OnceLock;
pub struct OnceLockTakeMapPayload {
    value: String,
}

impl OnceLockTakeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("once-lock-take-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("once-lock-take-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-once-lock-take-map:{}", self.value)
    }
}

pub fn selected_once_lock_take_map(raw: &str) -> String {
    let mut slot: OnceLock<OnceLockTakeMapPayload> = OnceLock::new();
    let _ = slot.set(OnceLockTakeMapPayload::new(raw));
    slot.take()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_once_lock_take_map(raw: &str) -> String {
    OnceLockTakeMapPayload::new(raw).unused_label()
}
