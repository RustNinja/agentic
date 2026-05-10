pub struct StrToUppercaseMapPayload {
    value: String,
}

impl StrToUppercaseMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-to-uppercase-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-to-uppercase-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-to-uppercase-map:{}", self.value)
    }
}

pub fn selected_str_to_uppercase_map(raw: &str) -> String {
    let value = raw.to_uppercase();
    StrToUppercaseMapPayload::new(&value).render_label()
}

pub fn dead_live_str_to_uppercase_map(raw: &str) -> String {
    StrToUppercaseMapPayload::new(raw).unused_label()
}
