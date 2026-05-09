use std::iter;

#[derive(Clone)]
pub struct IterSuccessorsMapPayload {
    value: String,
}

impl IterSuccessorsMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iter-successors-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("iter-successors-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iter-successors-map:{}", self.value)
    }
}

pub fn selected_iter_successors_map(raw: &str) -> String {
    iter::successors(Some(IterSuccessorsMapPayload::new(raw)), |_| None)
        .next()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "iter-successors-map:missing".to_string())
}

pub fn dead_live_iter_successors_map(raw: &str) -> String {
    IterSuccessorsMapPayload::new(raw).dead_method()
}
