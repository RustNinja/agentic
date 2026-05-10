use std::sync::Arc;
pub struct ArcTryUnwrapOkMapPayload {
    value: String,
}

impl ArcTryUnwrapOkMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("arc-try-unwrap-ok-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("arc-try-unwrap-ok-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-arc-try-unwrap-ok-map:{}", self.value)
    }
}

pub fn selected_arc_try_unwrap_ok_map(raw: &str) -> String {
    let payload = Arc::new(ArcTryUnwrapOkMapPayload::new(raw));
    Arc::try_unwrap(payload)
        .ok()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_arc_try_unwrap_ok_map(raw: &str) -> String {
    ArcTryUnwrapOkMapPayload::new(raw).unused_label()
}
