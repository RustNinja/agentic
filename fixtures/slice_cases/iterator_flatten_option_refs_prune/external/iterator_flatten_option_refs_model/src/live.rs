pub struct IteratorFlattenOptionRefsPayload {
    value: String,
}

impl IteratorFlattenOptionRefsPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-flatten-option-refs:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":bumped");
        self.render_label()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-flatten-option-refs:{}", self.value)
    }
}

pub fn selected_iterator_flatten_option_refs(raw: &str) -> String {
    let left = Some(IteratorFlattenOptionRefsPayload::new(raw));
    let right = None;
    [&left, &right]
        .into_iter()
        .flatten()
        .map(|payload| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_flatten_option_refs(raw: &str) -> String {
    IteratorFlattenOptionRefsPayload::new(raw).dead_method()
}
