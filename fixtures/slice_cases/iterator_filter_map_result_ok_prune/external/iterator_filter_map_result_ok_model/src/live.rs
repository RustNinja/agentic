pub struct IteratorFilterMapResultOkPayload {
    value: String,
}

impl IteratorFilterMapResultOkPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-filter-map-result-ok:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":bumped");
        self.render_label()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-filter-map-result-ok:{}", self.value)
    }
}

pub struct IteratorFilterMapResultOkError {
    code: String,
}

impl IteratorFilterMapResultOkError {
    pub fn new(raw: &str) -> Self {
        Self {
            code: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("iterator-filter-map-result-ok-error:{}", self.code)
    }

    pub fn dead_error_method(&self) -> String {
        format!("dead-iterator-filter-map-result-ok-error:{}", self.code)
    }
}

pub fn selected_iterator_filter_map_result_ok(raw: &str) -> String {
    let results: Vec<Result<IteratorFilterMapResultOkPayload, IteratorFilterMapResultOkError>> = vec![
        Ok(IteratorFilterMapResultOkPayload::new(raw)),
        Err(IteratorFilterMapResultOkError::new(raw)),
    ];
    results
        .into_iter()
        .filter_map(Result::ok)
        .map(|payload| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_filter_map_result_ok(raw: &str) -> String {
    IteratorFilterMapResultOkPayload::new(raw).dead_method()
}
