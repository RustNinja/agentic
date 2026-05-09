pub struct StrSplitAsciiWhitespaceMapPayload {
    value: String,
}

impl StrSplitAsciiWhitespaceMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-split-ascii-whitespace-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-split-ascii-whitespace-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-split-ascii-whitespace-map:{}", self.value)
    }
}

pub fn selected_str_split_ascii_whitespace_map(raw: &str) -> String {
    raw.split_ascii_whitespace()
        .map(|part| StrSplitAsciiWhitespaceMapPayload::new(part).render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_split_ascii_whitespace_map(raw: &str) -> String {
    StrSplitAsciiWhitespaceMapPayload::new(raw).unused_label()
}
