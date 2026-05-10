pub struct StringIntoBytesFirstMapPayload {
    value: String,
}

impl StringIntoBytesFirstMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-into-bytes-first-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-into-bytes-first-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-into-bytes-first-map:{}", self.value)
    }
}

pub fn selected_string_into_bytes_first_map(raw: &str) -> String {
    String::from(raw)
        .into_bytes()
        .into_iter()
        .next()
        .map(|byte| StringIntoBytesFirstMapPayload::new(&byte.to_string()).render_label())
        .unwrap_or_default()
}

pub fn dead_live_string_into_bytes_first_map(raw: &str) -> String {
    StringIntoBytesFirstMapPayload::new(raw).unused_label()
}
