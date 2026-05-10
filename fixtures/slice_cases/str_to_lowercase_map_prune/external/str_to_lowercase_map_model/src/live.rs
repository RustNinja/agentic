pub struct StrToLowercaseMapPayload {
    value: String,
}

impl StrToLowercaseMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-to-lowercase-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-to-lowercase-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-to-lowercase-map:{}", self.value)
    }
}

pub fn selected_str_to_lowercase_map(raw: &str) -> String {
    let value = raw.to_lowercase();
    StrToLowercaseMapPayload::new(&value).render_label()
}

pub fn dead_live_str_to_lowercase_map(raw: &str) -> String {
    StrToLowercaseMapPayload::new(raw).unused_label()
}
