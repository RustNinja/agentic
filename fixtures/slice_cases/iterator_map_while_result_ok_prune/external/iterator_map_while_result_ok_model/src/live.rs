pub struct IteratorMapWhileResultOkPayload {
    value: String,
}

impl IteratorMapWhileResultOkPayload {
    pub fn try_parse(raw: &str) -> Result<Self, IteratorMapWhileResultOkError> {
        if raw.trim().is_empty() {
            Err(IteratorMapWhileResultOkError::new(raw))
        } else {
            Ok(Self {
                value: raw.trim().to_string(),
            })
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-map-while-result-ok:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-map-while-result-ok:{}", self.value)
    }
}

pub struct IteratorMapWhileResultOkError {
    reason: String,
}

impl IteratorMapWhileResultOkError {
    pub fn new(raw: &str) -> Self {
        Self {
            reason: raw.to_string(),
        }
    }

    pub fn dead_error_method(&self) -> String {
        format!("dead-iterator-map-while-result-ok-error:{}", self.reason)
    }
}

pub fn selected_iterator_map_while_result_ok(raw: &str) -> String {
    raw.split(',')
        .map_while(|part| IteratorMapWhileResultOkPayload::try_parse(part).ok())
        .map(|payload| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_map_while_result_ok(raw: &str) -> String {
    IteratorMapWhileResultOkPayload::try_parse(raw)
        .map(|payload| payload.unused_label())
        .unwrap_or_else(|err| err.dead_error_method())
}
