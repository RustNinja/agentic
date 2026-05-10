pub struct StringSplitOffMapPayload {
    value: String,
}

impl StringSplitOffMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-split-off-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-split-off-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-split-off-map:{}", self.value)
    }
}

pub fn selected_string_split_off_map(raw: &str) -> String {
    let mut value = format!("head-{raw}");
    let tail = value.split_off(5);
    StringSplitOffMapPayload::new(&tail).render_label()
}

pub fn dead_live_string_split_off_map(raw: &str) -> String {
    StringSplitOffMapPayload::new(raw).unused_label()
}
