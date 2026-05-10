use std::sync::Arc;
pub struct ArcIntoInnerMapPayload {
    value: String,
}

impl ArcIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("arc-into-inner-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("arc-into-inner-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-arc-into-inner-map:{}", self.value)
    }
}

pub fn selected_arc_into_inner_map(raw: &str) -> String {
    let payload = Arc::new(ArcIntoInnerMapPayload::new(raw));
    Arc::into_inner(payload)
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_arc_into_inner_map(raw: &str) -> String {
    ArcIntoInnerMapPayload::new(raw).unused_label()
}
