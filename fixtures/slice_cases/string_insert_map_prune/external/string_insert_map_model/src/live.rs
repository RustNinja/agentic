pub struct StringInsertMapPayload {
    value: String,
}

impl StringInsertMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-insert-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-insert-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-insert-map:{}", self.value)
    }
}

pub fn selected_string_insert_map(raw: &str) -> String {
    let mut value = raw.to_string();
    value.insert(0, 'x');
    StringInsertMapPayload::new(&value).render_label()
}

pub fn dead_live_string_insert_map(raw: &str) -> String {
    StringInsertMapPayload::new(raw).unused_label()
}
