pub struct StrSplitOnceMapPayload {
    value: String,
}

impl StrSplitOnceMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-split-once-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-split-once-map:{}", self.value)
    }
}

pub fn selected_str_split_once_map(raw: &str) -> String {
    raw.split_once('=')
        .map(|(_, value)| StrSplitOnceMapPayload::new(value).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_split_once_map(raw: &str) -> String {
    StrSplitOnceMapPayload::new(raw).unused_label()
}
