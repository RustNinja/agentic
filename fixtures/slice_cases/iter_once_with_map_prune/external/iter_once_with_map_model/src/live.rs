use std::iter;
#[derive(Clone)]
pub struct IterOnceWithMapPayload {
    value: String,
}

impl IterOnceWithMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iter-once-with-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("iter-once-with-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iter-once-with-map:{}", self.value)
    }
}

pub fn selected_iter_once_with_map(raw: &str) -> String {
    iter::once_with(|| IterOnceWithMapPayload::new(raw))
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_iter_once_with_map(raw: &str) -> String {
    IterOnceWithMapPayload::new(raw).unused_label()
}
