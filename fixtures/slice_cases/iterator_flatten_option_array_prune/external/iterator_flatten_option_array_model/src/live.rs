pub struct IteratorFlattenOptionArrayPayload {
    value: String,
}

impl IteratorFlattenOptionArrayPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-flatten-option-array:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":bumped");
        self.render_label()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-flatten-option-array:{}", self.value)
    }
}

pub fn selected_iterator_flatten_option_array(raw: &str) -> String {
    [Some(IteratorFlattenOptionArrayPayload::new(raw)), None]
        .into_iter()
        .flatten()
        .map(|payload| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_flatten_option_array(raw: &str) -> String {
    IteratorFlattenOptionArrayPayload::new(raw).dead_method()
}
