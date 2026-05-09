pub struct StrSplitWhitespaceFindMapPayload {
    value: String,
}

impl StrSplitWhitespaceFindMapPayload {
    pub fn try_parse(raw: &str) -> Result<Self, StrSplitWhitespaceFindMapError> {
        if raw.trim().is_empty() {
            Err(StrSplitWhitespaceFindMapError::new(raw))
        } else {
            Ok(Self {
                value: raw.trim().to_string(),
            })
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-split-whitespace-find-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-split-whitespace-find-map:{}", self.value)
    }
}

pub struct StrSplitWhitespaceFindMapError {
    reason: String,
}

impl StrSplitWhitespaceFindMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            reason: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("str-split-whitespace-find-map-error:{}", self.reason)
    }
}

pub fn selected_str_split_whitespace_find_map(raw: &str) -> String {
    raw.split_whitespace()
        .find_map(|part| {
            StrSplitWhitespaceFindMapPayload::try_parse(part)
                .ok()
                .map(|payload| payload.render_label())
        })
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_split_whitespace_find_map(raw: &str) -> String {
    StrSplitWhitespaceFindMapPayload::try_parse(raw)
        .map(|payload| payload.unused_label())
        .unwrap_or_else(|err| err.render_error())
}
