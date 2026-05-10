pub struct StringTruncateMapPayload {
    value: String,
}

impl StringTruncateMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-truncate-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-truncate-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-truncate-map:{}", self.value)
    }
}

pub fn selected_string_truncate_map(raw: &str) -> String {
    let mut value = format!("{raw}-tail");
    value.truncate(raw.len());
    StringTruncateMapPayload::new(&value).render_label()
}

pub fn dead_live_string_truncate_map(raw: &str) -> String {
    StringTruncateMapPayload::new(raw).unused_label()
}
