pub struct StringClearPushStrMapPayload {
    value: String,
}

impl StringClearPushStrMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-clear-push-str-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-clear-push-str-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-clear-push-str-map:{}", self.value)
    }
}

pub fn selected_string_clear_push_str_map(raw: &str) -> String {
    let mut value = String::from("dead");
    value.clear();
    value.push_str(raw);
    StringClearPushStrMapPayload::new(&value).render_label()
}

pub fn dead_live_string_clear_push_str_map(raw: &str) -> String {
    StringClearPushStrMapPayload::new(raw).unused_label()
}
