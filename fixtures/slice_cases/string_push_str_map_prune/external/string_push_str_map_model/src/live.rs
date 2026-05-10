pub struct StringPushStrMapPayload {
    value: String,
}

impl StringPushStrMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-push-str-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-push-str-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-push-str-map:{}", self.value)
    }
}

pub fn selected_string_push_str_map(raw: &str) -> String {
    let mut value = String::from(raw);
    value.push_str("-tail");
    StringPushStrMapPayload::new(&value).render_label()
}

pub fn dead_live_string_push_str_map(raw: &str) -> String {
    StringPushStrMapPayload::new(raw).unused_label()
}
