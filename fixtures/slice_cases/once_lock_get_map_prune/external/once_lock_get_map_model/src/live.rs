use std::sync::OnceLock;

#[derive(Clone)]
pub struct OnceLockGetMapPayload {
    value: String,
}

impl OnceLockGetMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("once-lock-get-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("once-lock-get-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-once-lock-get-map:{}", self.value)
    }
}

pub fn selected_once_lock_get_map(raw: &str) -> String {
    let lock = OnceLock::new();
    let _ = lock.set(OnceLockGetMapPayload::new(raw));
    lock.get()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "once-lock-get-map:missing".to_string())
}

pub fn dead_live_once_lock_get_map(raw: &str) -> String {
    OnceLockGetMapPayload::new(raw).dead_method()
}
