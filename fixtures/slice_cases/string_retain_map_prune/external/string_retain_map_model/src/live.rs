pub struct StringRetainMapPayload {
    value: String,
}

impl StringRetainMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-retain-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-retain-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-retain-map:{}", self.value)
    }
}

pub fn selected_string_retain_map(raw: &str) -> String {
    let mut value = format!("a-{raw}");
    value.retain(|ch| ch != '-');
    StringRetainMapPayload::new(&value).render_label()
}

pub fn dead_live_string_retain_map(raw: &str) -> String {
    StringRetainMapPayload::new(raw).unused_label()
}
