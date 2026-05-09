pub struct StringRemoveCharMapPayload {
    value: String,
}

impl StringRemoveCharMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-remove-char-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-remove-char-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-remove-char-map:{}", self.value)
    }
}

pub fn selected_string_remove_char_map(raw: &str) -> String {
    let mut value = if raw.is_empty() {
        "x".to_string()
    } else {
        raw.to_string()
    };
    let ch = value.remove(0);
    StringRemoveCharMapPayload::new(&ch.to_string()).render_label()
}

pub fn dead_live_string_remove_char_map(raw: &str) -> String {
    StringRemoveCharMapPayload::new(raw).unused_label()
}
