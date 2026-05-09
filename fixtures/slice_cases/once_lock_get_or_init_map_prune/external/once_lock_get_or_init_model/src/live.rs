use std::sync::OnceLock;

pub struct OnceLockGetOrInitMapPayload {
    value: String,
}

impl OnceLockGetOrInitMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("once-lock-get-or-init-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("once-lock-get-or-init-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-once-lock-get-or-init-map:{}", self.value)
    }
}

pub fn selected_once_lock_get_or_init_map(raw: &str) -> String {
    let payload = OnceLock::new();
    payload
        .get_or_init(|| OnceLockGetOrInitMapPayload::new(raw))
        .render_label()
}

pub fn dead_live_once_lock_get_or_init_map(raw: &str) -> String {
    OnceLockGetOrInitMapPayload::new(raw).dead_method()
}
