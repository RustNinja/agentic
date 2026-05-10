pub struct StringShrinkToFitMapPayload {
    value: String,
}

impl StringShrinkToFitMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-shrink-to-fit-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-shrink-to-fit-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-shrink-to-fit-map:{}", self.value)
    }
}

pub fn selected_string_shrink_to_fit_map(raw: &str) -> String {
    let mut value = raw.to_string();
    value.push_str("-tail");
    value.shrink_to_fit();
    StringShrinkToFitMapPayload::new(&value).render_label()
}

pub fn dead_live_string_shrink_to_fit_map(raw: &str) -> String {
    StringShrinkToFitMapPayload::new(raw).unused_label()
}
