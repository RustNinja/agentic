pub struct IteratorFindMapResultOkPayload {
    value: String,
}

impl IteratorFindMapResultOkPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-find-map-result-ok:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":bumped");
        self.render_label()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-find-map-result-ok:{}", self.value)
    }
}

pub struct IteratorFindMapResultOkError {
    code: String,
}

impl IteratorFindMapResultOkError {
    pub fn new(raw: &str) -> Self {
        Self {
            code: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("iterator-find-map-result-ok-error:{}", self.code)
    }

    pub fn dead_error_method(&self) -> String {
        format!("dead-iterator-find-map-result-ok-error:{}", self.code)
    }
}

pub fn selected_iterator_find_map_result_ok(raw: &str) -> String {
    let results: Vec<Result<IteratorFindMapResultOkPayload, IteratorFindMapResultOkError>> = vec![
        Err(IteratorFindMapResultOkError::new(raw)),
        Ok(IteratorFindMapResultOkPayload::new(raw)),
    ];
    results
        .into_iter()
        .find_map(Result::ok)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "iterator-find-map-result-ok:missing".to_string())
}

pub fn dead_live_iterator_find_map_result_ok(raw: &str) -> String {
    IteratorFindMapResultOkPayload::new(raw).dead_method()
}
