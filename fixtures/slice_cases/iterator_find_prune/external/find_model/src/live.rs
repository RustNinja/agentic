pub struct FindPayload {
    value: String,
}

impl FindPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn accepts(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn render_label(&self) -> String {
        format!("find:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-find:{}", self.value)
    }
}

fn find_items(raw: &str) -> Vec<FindPayload> {
    vec![FindPayload::new(raw)]
}

pub fn selected_find(raw: &str) -> String {
    find_items(raw)
        .iter()
        .find(|payload| payload.accepts())
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "find:missing".to_string())
}

pub fn dead_live_find(raw: &str) -> String {
    FindPayload::new(raw).dead_method()
}
