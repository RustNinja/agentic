use std::sync::LazyLock;
pub struct LazyLockForceMapPayload {
    value: String,
}

impl LazyLockForceMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("lazy-lock-force-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("lazy-lock-force-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-lazy-lock-force-map:{}", self.value)
    }
}

pub fn selected_lazy_lock_force_map(raw: &str) -> String {
    static PAYLOAD: LazyLock<LazyLockForceMapPayload> =
        LazyLock::new(|| LazyLockForceMapPayload::new("lazy"));
    PAYLOAD.render_label()
}

pub fn dead_live_lazy_lock_force_map(raw: &str) -> String {
    LazyLockForceMapPayload::new(raw).unused_label()
}
