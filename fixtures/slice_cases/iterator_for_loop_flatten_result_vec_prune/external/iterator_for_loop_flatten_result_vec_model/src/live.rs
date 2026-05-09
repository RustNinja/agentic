pub struct IteratorForLoopFlattenResultVecPayload {
    value: String,
}

impl IteratorForLoopFlattenResultVecPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-for-loop-flatten-result-vec:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":bumped");
        self.render_label()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-for-loop-flatten-result-vec:{}", self.value)
    }
}

pub struct IteratorForLoopFlattenResultVecError {
    code: String,
}

impl IteratorForLoopFlattenResultVecError {
    pub fn new(raw: &str) -> Self {
        Self {
            code: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("iterator-for-loop-flatten-result-vec-error:{}", self.code)
    }

    pub fn dead_error_method(&self) -> String {
        format!(
            "dead-iterator-for-loop-flatten-result-vec-error:{}",
            self.code
        )
    }
}

pub fn selected_iterator_for_loop_flatten_result_vec(raw: &str) -> String {
    let results: Vec<
        Result<IteratorForLoopFlattenResultVecPayload, IteratorForLoopFlattenResultVecError>,
    > = vec![
        Ok(IteratorForLoopFlattenResultVecPayload::new(raw)),
        Err(IteratorForLoopFlattenResultVecError::new(raw)),
    ];
    let mut rendered = Vec::new();
    for payload in results.into_iter().flatten() {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

pub fn dead_live_iterator_for_loop_flatten_result_vec(raw: &str) -> String {
    IteratorForLoopFlattenResultVecPayload::new(raw).dead_method()
}
