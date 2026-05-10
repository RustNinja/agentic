pub struct StringInsertStrMapPayload {
    value: String,
}

impl StringInsertStrMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-insert-str-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-insert-str-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-insert-str-map:{}", self.value)
    }
}

pub fn selected_string_insert_str_map(raw: &str) -> String {
    let mut value = raw.to_string();
    value.insert_str(0, "head-");
    StringInsertStrMapPayload::new(&value).render_label()
}

pub fn dead_live_string_insert_str_map(raw: &str) -> String {
    StringInsertStrMapPayload::new(raw).unused_label()
}
