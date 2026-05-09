pub struct IteratorFlattenResultVecPayload {
    value: String,
}

impl IteratorFlattenResultVecPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-flatten-result-vec:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":bumped");
        self.render_label()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-flatten-result-vec:{}", self.value)
    }
}

pub struct IteratorFlattenResultVecError {
    code: String,
}

impl IteratorFlattenResultVecError {
    pub fn new(raw: &str) -> Self {
        Self {
            code: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("iterator-flatten-result-vec-error:{}", self.code)
    }

    pub fn dead_error_method(&self) -> String {
        format!("dead-iterator-flatten-result-vec-error:{}", self.code)
    }
}

pub fn selected_iterator_flatten_result_vec(raw: &str) -> String {
    let results: Vec<Result<IteratorFlattenResultVecPayload, IteratorFlattenResultVecError>> = vec![
        Ok(IteratorFlattenResultVecPayload::new(raw)),
        Err(IteratorFlattenResultVecError::new(raw)),
    ];
    results
        .into_iter()
        .flatten()
        .map(|payload| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_flatten_result_vec(raw: &str) -> String {
    IteratorFlattenResultVecPayload::new(raw).dead_method()
}
