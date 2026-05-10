use std::sync::RwLock;
#[derive(Debug)]
pub struct RwlockIntoInnerUnwrapMapPayload {
    value: String,
}

impl RwlockIntoInnerUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rwlock-into-inner-unwrap-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("rwlock-into-inner-unwrap-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-rwlock-into-inner-unwrap-map:{}", self.value)
    }
}

pub fn selected_rwlock_into_inner_unwrap_map(raw: &str) -> String {
    let payload = RwLock::new(RwlockIntoInnerUnwrapMapPayload::new(raw));
    RwLock::into_inner(payload).unwrap().render_label()
}

pub fn dead_live_rwlock_into_inner_unwrap_map(raw: &str) -> String {
    RwlockIntoInnerUnwrapMapPayload::new(raw).unused_label()
}
