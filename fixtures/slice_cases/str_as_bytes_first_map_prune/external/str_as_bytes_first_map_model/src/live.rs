pub struct StrAsBytesFirstMapPayload {
    value: String,
}

impl StrAsBytesFirstMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-as-bytes-first-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-as-bytes-first-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-as-bytes-first-map:{}", self.value)
    }
}

pub fn selected_str_as_bytes_first_map(raw: &str) -> String {
    raw.as_bytes()
        .first()
        .map(|byte| StrAsBytesFirstMapPayload::new(&byte.to_string()).render_label())
        .unwrap_or_default()
}

pub fn dead_live_str_as_bytes_first_map(raw: &str) -> String {
    StrAsBytesFirstMapPayload::new(raw).unused_label()
}
