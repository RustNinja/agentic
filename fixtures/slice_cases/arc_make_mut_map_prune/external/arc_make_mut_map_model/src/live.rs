use std::sync::Arc;

#[derive(Clone)]
pub struct ArcMakeMutMapPayload {
    value: String,
}

impl ArcMakeMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("arc-make-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("arc-make-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-arc-make-mut-map:{}", self.value)
    }
}

pub fn selected_arc_make_mut_map(raw: &str) -> String {
    let mut payload = Arc::new(ArcMakeMutMapPayload::new(raw));
    Arc::make_mut(&mut payload).bump_and_render()
}

pub fn dead_live_arc_make_mut_map(raw: &str) -> String {
    ArcMakeMutMapPayload::new(raw).dead_method()
}
