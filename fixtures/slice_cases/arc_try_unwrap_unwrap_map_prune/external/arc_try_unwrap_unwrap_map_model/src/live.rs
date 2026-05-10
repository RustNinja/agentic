use std::sync::Arc;
#[derive(Debug)]
pub struct ArcTryUnwrapUnwrapMapPayload {
    value: String,
}

impl ArcTryUnwrapUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("arc-try-unwrap-unwrap-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("arc-try-unwrap-unwrap-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-arc-try-unwrap-unwrap-map:{}", self.value)
    }
}

pub fn selected_arc_try_unwrap_unwrap_map(raw: &str) -> String {
    let payload = Arc::new(ArcTryUnwrapUnwrapMapPayload::new(raw));
    Arc::try_unwrap(payload).unwrap().render_label()
}

pub fn dead_live_arc_try_unwrap_unwrap_map(raw: &str) -> String {
    ArcTryUnwrapUnwrapMapPayload::new(raw).unused_label()
}
