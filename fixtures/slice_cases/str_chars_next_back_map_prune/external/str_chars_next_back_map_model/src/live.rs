pub struct StrCharsNextBackMapPayload {
    value: String,
}

impl StrCharsNextBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-chars-next-back-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-chars-next-back-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-chars-next-back-map:{}", self.value)
    }
}

pub fn selected_str_chars_next_back_map(raw: &str) -> String {
    raw.chars()
        .next_back()
        .map(|ch| StrCharsNextBackMapPayload::new(&ch.to_string()).render_label())
        .unwrap_or_default()
}

pub fn dead_live_str_chars_next_back_map(raw: &str) -> String {
    StrCharsNextBackMapPayload::new(raw).unused_label()
}
