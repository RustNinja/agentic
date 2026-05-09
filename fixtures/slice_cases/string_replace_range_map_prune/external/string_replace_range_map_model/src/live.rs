pub struct StringReplaceRangeMapPayload {
    value: String,
}

impl StringReplaceRangeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-replace-range-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-replace-range-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-replace-range-map:{}", self.value)
    }
}

pub fn selected_string_replace_range_map(raw: &str) -> String {
    let mut value = raw.to_string();
    value.replace_range(..0, "head-");
    StringReplaceRangeMapPayload::new(&value).render_label()
}

pub fn dead_live_string_replace_range_map(raw: &str) -> String {
    StringReplaceRangeMapPayload::new(raw).unused_label()
}
