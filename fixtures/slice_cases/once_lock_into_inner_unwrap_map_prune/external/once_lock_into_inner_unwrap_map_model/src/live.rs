use std::sync::OnceLock;
#[derive(Debug)]
pub struct OnceLockIntoInnerUnwrapMapPayload {
    value: String,
}

impl OnceLockIntoInnerUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("once-lock-into-inner-unwrap-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("once-lock-into-inner-unwrap-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-once-lock-into-inner-unwrap-map:{}", self.value)
    }
}

pub fn selected_once_lock_into_inner_unwrap_map(raw: &str) -> String {
    let slot: OnceLock<OnceLockIntoInnerUnwrapMapPayload> = OnceLock::new();
    let _ = slot.set(OnceLockIntoInnerUnwrapMapPayload::new(raw));
    slot.into_inner().unwrap().render_label()
}

pub fn dead_live_once_lock_into_inner_unwrap_map(raw: &str) -> String {
    OnceLockIntoInnerUnwrapMapPayload::new(raw).unused_label()
}
