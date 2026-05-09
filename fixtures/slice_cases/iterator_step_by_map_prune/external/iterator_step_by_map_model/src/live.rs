#[derive(Clone)]
pub struct IteratorStepByMapPayload {
    value: String,
}

impl IteratorStepByMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-step-by-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("iterator-step-by-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-step-by-map:{}", self.value)
    }
}

pub fn selected_iterator_step_by_map(raw: &str) -> String {
    let items = vec![
        IteratorStepByMapPayload::new(raw),
        IteratorStepByMapPayload::new("dead"),
    ];
    items
        .iter()
        .step_by(2)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| format!("iterator-step-by-map:missing"))
}

pub fn dead_live_iterator_step_by_map(raw: &str) -> String {
    let mut payload = IteratorStepByMapPayload::new(raw);
    payload.bump_and_render()
}
