use std::sync::Arc;

pub struct ArcAsRefMapPayload {
    value: String,
}

impl ArcAsRefMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("arc-as-ref-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("arc-as-ref-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-arc-as-ref-map:{}", self.value)
    }
}

pub fn selected_arc_as_ref_map(raw: &str) -> String {
    let payload = Arc::new(ArcAsRefMapPayload::new(raw));
    payload.as_ref().render_label()
}

pub fn dead_live_arc_as_ref_map(raw: &str) -> String {
    ArcAsRefMapPayload::new(raw).dead_method()
}
