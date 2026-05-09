pub struct StringPopCharMapPayload {
    value: String,
}

impl StringPopCharMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-pop-char-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-pop-char-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-pop-char-map:{}", self.value)
    }
}

pub fn selected_string_pop_char_map(raw: &str) -> String {
    let mut value = raw.to_string();
    value
        .pop()
        .map(|ch| StringPopCharMapPayload::new(&ch.to_string()).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_string_pop_char_map(raw: &str) -> String {
    StringPopCharMapPayload::new(raw).unused_label()
}
