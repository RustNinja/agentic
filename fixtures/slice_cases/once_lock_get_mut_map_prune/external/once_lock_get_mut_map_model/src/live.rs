use std::sync::OnceLock;
pub struct OnceLockGetMutMapPayload {
    value: String,
}

impl OnceLockGetMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("once-lock-get-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("once-lock-get-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-once-lock-get-mut-map:{}", self.value)
    }
}

pub fn selected_once_lock_get_mut_map(raw: &str) -> String {
    let mut slot: OnceLock<OnceLockGetMutMapPayload> = OnceLock::new();
    let _ = slot.set(OnceLockGetMutMapPayload::new(raw));
    slot.get_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_default()
}

pub fn dead_live_once_lock_get_mut_map(raw: &str) -> String {
    OnceLockGetMutMapPayload::new(raw).unused_label()
}
