pub struct IteratorFilterMapMatchOptionQuestionRequest {
    method: &'static str,
    raw: String,
}

impl IteratorFilterMapMatchOptionQuestionRequest {
    pub fn live(raw: &str) -> Self {
        Self {
            method: "live",
            raw: raw.to_string(),
        }
    }

    pub fn skipped(raw: &str) -> Self {
        Self {
            method: "skipped",
            raw: raw.to_string(),
        }
    }

    pub fn method(&self) -> &str {
        self.method
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }
}

pub struct IteratorFilterMapMatchOptionQuestionPayload {
    value: String,
}

impl IteratorFilterMapMatchOptionQuestionPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-filter-map-match-option-question:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!(
            "dead-iterator-filter-map-match-option-question:{}",
            self.value
        )
    }
}

fn lookup_payload(raw: &str) -> Option<IteratorFilterMapMatchOptionQuestionPayload> {
    (!raw.trim().is_empty()).then(|| IteratorFilterMapMatchOptionQuestionPayload::new(raw))
}

pub fn selected_iterator_filter_map_match_option_question(raw: &str) -> String {
    let requests = [
        IteratorFilterMapMatchOptionQuestionRequest::live(raw),
        IteratorFilterMapMatchOptionQuestionRequest::skipped(raw),
    ];
    requests
        .iter()
        .filter_map(|request| match request.method() {
            "live" => {
                let payload = lookup_payload(request.raw())?;
                Some(payload.render_label())
            }
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_filter_map_match_option_question(raw: &str) -> String {
    lookup_payload(raw)
        .map(|payload| payload.unused_label())
        .unwrap_or_else(|| "dead".to_string())
}
