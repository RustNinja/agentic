pub struct IteratorFilterMapMatchResultOkQuestionRequest {
    method: &'static str,
    raw: String,
}

impl IteratorFilterMapMatchResultOkQuestionRequest {
    pub fn live(raw: &str) -> Self {
        Self {
            method: "live",
            raw: raw.to_string(),
        }
    }

    pub fn ignored(raw: &str) -> Self {
        Self {
            method: "ignored",
            raw: raw.to_string(),
        }
    }

    pub fn method(&self) -> &str {
        self.method
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn dead_request_method(&self) -> String {
        format!(
            "dead-iterator-filter-map-match-result-ok-question-request:{}",
            self.raw
        )
    }
}

pub struct IteratorFilterMapMatchResultOkQuestionPayload {
    value: String,
}

impl IteratorFilterMapMatchResultOkQuestionPayload {
    pub fn try_parse(raw: &str) -> Result<Self, IteratorFilterMapMatchResultOkQuestionError> {
        if raw.trim().is_empty() {
            Err(IteratorFilterMapMatchResultOkQuestionError::new(raw))
        } else {
            Ok(Self {
                value: raw.trim().to_string(),
            })
        }
    }

    pub fn render_label(&self) -> String {
        format!(
            "iterator-filter-map-match-result-ok-question:{}",
            self.value
        )
    }

    pub fn unused_label(&self) -> String {
        format!(
            "dead-iterator-filter-map-match-result-ok-question:{}",
            self.value
        )
    }
}

pub struct IteratorFilterMapMatchResultOkQuestionError {
    reason: String,
}

impl IteratorFilterMapMatchResultOkQuestionError {
    pub fn new(raw: &str) -> Self {
        Self {
            reason: raw.to_string(),
        }
    }

    pub fn dead_error_method(&self) -> String {
        format!(
            "dead-iterator-filter-map-match-result-ok-question-error:{}",
            self.reason
        )
    }
}

pub fn selected_iterator_filter_map_match_result_ok_question(raw: &str) -> String {
    let requests = [
        IteratorFilterMapMatchResultOkQuestionRequest::live(raw),
        IteratorFilterMapMatchResultOkQuestionRequest::ignored(raw),
    ];
    requests
        .iter()
        .filter_map(|request| match request.method() {
            "live" => Some(
                IteratorFilterMapMatchResultOkQuestionPayload::try_parse(request.raw())
                    .ok()?
                    .render_label(),
            ),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_filter_map_match_result_ok_question(raw: &str) -> String {
    IteratorFilterMapMatchResultOkQuestionRequest::ignored(raw).dead_request_method()
}
