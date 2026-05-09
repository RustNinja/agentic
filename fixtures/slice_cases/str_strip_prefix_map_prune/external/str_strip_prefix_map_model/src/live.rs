pub struct StrStripPrefixMapPayload {
    value: String,
}

impl StrStripPrefixMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-strip-prefix-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-strip-prefix-map:{}", self.value)
    }
}

pub fn selected_str_strip_prefix_map(raw: &str) -> String {
    raw.strip_prefix("live:")
        .map(StrStripPrefixMapPayload::new)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_strip_prefix_map(raw: &str) -> String {
    StrStripPrefixMapPayload::new(raw).unused_label()
}
