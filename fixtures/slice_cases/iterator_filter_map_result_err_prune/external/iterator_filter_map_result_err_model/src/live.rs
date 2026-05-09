pub struct IteratorFilterMapResultErrPayload {
    value: String,
}

impl IteratorFilterMapResultErrPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-filter-map-result-err:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":bumped");
        self.render_label()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-filter-map-result-err:{}", self.value)
    }
}

pub struct IteratorFilterMapResultErrError {
    code: String,
}

impl IteratorFilterMapResultErrError {
    pub fn new(raw: &str) -> Self {
        Self {
            code: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("iterator-filter-map-result-err-error:{}", self.code)
    }

    pub fn dead_error_method(&self) -> String {
        format!("dead-iterator-filter-map-result-err-error:{}", self.code)
    }
}

pub fn selected_iterator_filter_map_result_err(raw: &str) -> String {
    let results: Vec<Result<IteratorFilterMapResultErrPayload, IteratorFilterMapResultErrError>> = vec![
        Ok(IteratorFilterMapResultErrPayload::new(raw)),
        Err(IteratorFilterMapResultErrError::new(raw)),
    ];
    results
        .into_iter()
        .filter_map(Result::err)
        .map(|err| err.render_error())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_filter_map_result_err(raw: &str) -> String {
    IteratorFilterMapResultErrPayload::new(raw).dead_method()
}
