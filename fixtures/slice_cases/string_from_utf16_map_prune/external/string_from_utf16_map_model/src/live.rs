pub struct StringFromUtf16MapPayload {
    value: String,
}

impl StringFromUtf16MapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-from-utf16-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-from-utf16-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-from-utf16-map:{}", self.value)
    }
}

pub fn selected_string_from_utf16_map(raw: &str) -> String {
    let units = raw.encode_utf16().collect::<Vec<_>>();
    String::from_utf16(&units)
        .ok()
        .map(|value| StringFromUtf16MapPayload::new(&value).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_string_from_utf16_map(raw: &str) -> String {
    StringFromUtf16MapPayload::new(raw).unused_label()
}
