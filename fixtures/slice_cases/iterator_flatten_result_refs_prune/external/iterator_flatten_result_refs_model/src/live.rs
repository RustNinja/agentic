pub struct IteratorFlattenResultRefsPayload {
    value: String,
}

impl IteratorFlattenResultRefsPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-flatten-result-refs:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":bumped");
        self.render_label()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-flatten-result-refs:{}", self.value)
    }
}

pub struct IteratorFlattenResultRefsError {
    code: String,
}

impl IteratorFlattenResultRefsError {
    pub fn new(raw: &str) -> Self {
        Self {
            code: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("iterator-flatten-result-refs-error:{}", self.code)
    }

    pub fn dead_error_method(&self) -> String {
        format!("dead-iterator-flatten-result-refs-error:{}", self.code)
    }
}

pub fn selected_iterator_flatten_result_refs(raw: &str) -> String {
    let left: Result<IteratorFlattenResultRefsPayload, IteratorFlattenResultRefsError> =
        Ok(IteratorFlattenResultRefsPayload::new(raw));
    let right: Result<IteratorFlattenResultRefsPayload, IteratorFlattenResultRefsError> =
        Err(IteratorFlattenResultRefsError::new(raw));
    [&left, &right]
        .into_iter()
        .flatten()
        .map(|payload| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_flatten_result_refs(raw: &str) -> String {
    IteratorFlattenResultRefsPayload::new(raw).dead_method()
}
