pub struct IteratorFindMapNestedOkQuestionPayload {
    value: String,
}

impl IteratorFindMapNestedOkQuestionPayload {
    pub fn try_parse(raw: &str) -> Result<Self, IteratorFindMapNestedOkQuestionError> {
        if raw.trim().is_empty() {
            Err(IteratorFindMapNestedOkQuestionError::new(raw))
        } else {
            Ok(Self {
                value: raw.trim().to_string(),
            })
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-find-map-nested-ok-question:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-find-map-nested-ok-question:{}", self.value)
    }
}

pub struct IteratorFindMapNestedOkQuestionError {
    reason: String,
}

impl IteratorFindMapNestedOkQuestionError {
    pub fn new(raw: &str) -> Self {
        Self {
            reason: raw.to_string(),
        }
    }

    pub fn dead_error_method(&self) -> String {
        format!(
            "dead-iterator-find-map-nested-ok-question-error:{}",
            self.reason
        )
    }
}

pub fn selected_iterator_find_map_nested_ok_question(raw: &str) -> String {
    raw.split(',')
        .find_map(|part| {
            let payload = IteratorFindMapNestedOkQuestionPayload::try_parse(part).ok()?;
            Some(payload.render_label())
        })
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_iterator_find_map_nested_ok_question(raw: &str) -> String {
    IteratorFindMapNestedOkQuestionPayload::try_parse(raw)
        .map(|payload| payload.unused_label())
        .unwrap_or_else(|_| "dead".to_string())
}
