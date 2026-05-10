use std::sync::Arc;
pub struct ArcGetMutMapPayload {
    value: String,
}

impl ArcGetMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("arc-get-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("arc-get-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-arc-get-mut-map:{}", self.value)
    }
}

pub fn selected_arc_get_mut_map(raw: &str) -> String {
    let mut payload = Arc::new(ArcGetMutMapPayload::new(raw));
    Arc::get_mut(&mut payload)
        .map(|payload| payload.bump_and_render())
        .unwrap_or_default()
}

pub fn dead_live_arc_get_mut_map(raw: &str) -> String {
    ArcGetMutMapPayload::new(raw).unused_label()
}
