pub struct StringFromUtf8MapPayload {
    value: String,
}

impl StringFromUtf8MapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-from-utf8-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-from-utf8-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-from-utf8-map:{}", self.value)
    }
}

pub fn selected_string_from_utf8_map(raw: &str) -> String {
    String::from_utf8(raw.as_bytes().to_vec())
        .ok()
        .map(|value| StringFromUtf8MapPayload::new(&value).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_string_from_utf8_map(raw: &str) -> String {
    StringFromUtf8MapPayload::new(raw).unused_label()
}
