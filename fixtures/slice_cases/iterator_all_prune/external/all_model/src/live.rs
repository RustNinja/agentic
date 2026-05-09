pub struct AllPayload {
    value: String,
}

impl AllPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn accepts(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn render_label(&self) -> String {
        format!("all:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-all:{}", self.value)
    }
}

fn all_items(raw: &str) -> Vec<AllPayload> {
    vec![AllPayload::new(raw)]
}

pub fn selected_all(raw: &str) -> String {
    if all_items(raw).iter().all(|payload| payload.accepts()) {
        AllPayload::new(raw).render_label()
    } else {
        "all:rejected".to_string()
    }
}

pub fn dead_live_all(raw: &str) -> String {
    AllPayload::new(raw).dead_method()
}
