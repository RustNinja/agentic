pub struct StrReplaceMapPayload {
    value: String,
}

impl StrReplaceMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-replace-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-replace-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-replace-map:{}", self.value)
    }
}

pub fn selected_str_replace_map(raw: &str) -> String {
    let value = raw.replace('a', "b");
    StrReplaceMapPayload::new(&value).render_label()
}

pub fn dead_live_str_replace_map(raw: &str) -> String {
    StrReplaceMapPayload::new(raw).unused_label()
}
