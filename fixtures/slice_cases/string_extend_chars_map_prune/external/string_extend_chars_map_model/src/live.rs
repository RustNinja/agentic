pub struct StringExtendCharsMapPayload {
    value: String,
}

impl StringExtendCharsMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-extend-chars-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-extend-chars-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-extend-chars-map:{}", self.value)
    }
}

pub fn selected_string_extend_chars_map(raw: &str) -> String {
    let mut value = String::from("head-");
    value.extend(raw.chars());
    StringExtendCharsMapPayload::new(&value).render_label()
}

pub fn dead_live_string_extend_chars_map(raw: &str) -> String {
    StringExtendCharsMapPayload::new(raw).unused_label()
}
