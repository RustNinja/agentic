pub struct IteratorFlattenOptionResultPayload {
    value: String,
}

impl IteratorFlattenOptionResultPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-flatten-option-result:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-flatten-option-result:{}", self.value)
    }
}

pub struct IteratorFlattenOptionResultError;

pub fn selected_iterator_flatten_option_result(raw: &str) -> String {
    let items = vec![
        Some(Ok(IteratorFlattenOptionResultPayload::new(raw))),
        None,
        Some(Err(IteratorFlattenOptionResultError)),
    ];
    items
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|payload| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_flatten_option_result(raw: &str) -> String {
    IteratorFlattenOptionResultPayload::new(raw).unused_label()
}
