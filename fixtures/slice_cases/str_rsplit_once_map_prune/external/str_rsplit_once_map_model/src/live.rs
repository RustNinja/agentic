pub struct StrRsplitOnceMapPayload {
    value: String,
}

impl StrRsplitOnceMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-rsplit-once-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-rsplit-once-map:{}", self.value)
    }
}

pub fn selected_str_rsplit_once_map(raw: &str) -> String {
    raw.rsplit_once('/')
        .map(|(_, value)| StrRsplitOnceMapPayload::new(value).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_rsplit_once_map(raw: &str) -> String {
    StrRsplitOnceMapPayload::new(raw).unused_label()
}
