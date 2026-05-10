pub struct StringPushCharMapPayload {
    value: String,
}

impl StringPushCharMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-push-char-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-push-char-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-push-char-map:{}", self.value)
    }
}

pub fn selected_string_push_char_map(raw: &str) -> String {
    let mut value = String::new();
    value.push('x');
    value.push_str(raw);
    StringPushCharMapPayload::new(&value).render_label()
}

pub fn dead_live_string_push_char_map(raw: &str) -> String {
    StringPushCharMapPayload::new(raw).unused_label()
}
