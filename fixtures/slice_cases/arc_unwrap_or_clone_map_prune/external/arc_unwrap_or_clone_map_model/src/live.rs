use std::sync::Arc;
#[derive(Clone)]
pub struct ArcUnwrapOrCloneMapPayload {
    value: String,
}

impl ArcUnwrapOrCloneMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("arc-unwrap-or-clone-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("arc-unwrap-or-clone-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-arc-unwrap-or-clone-map:{}", self.value)
    }
}

pub fn selected_arc_unwrap_or_clone_map(raw: &str) -> String {
    let payload = Arc::new(ArcUnwrapOrCloneMapPayload::new(raw));
    Arc::unwrap_or_clone(payload).render_label()
}

pub fn dead_live_arc_unwrap_or_clone_map(raw: &str) -> String {
    ArcUnwrapOrCloneMapPayload::new(raw).unused_label()
}
