pub struct FreePayload {
    label: String,
}

impl FreePayload {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.trim().to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("free-closure:{}", self.label)
    }

    pub fn dead_method(self) -> String {
        format!("dead-free-closure:{}", self.label)
    }
}

pub fn with_free_payload(raw: &str, render: impl FnOnce(FreePayload) -> String) -> String {
    render(FreePayload::new(raw))
}

pub fn dead_live_free_closure(raw: &str) -> String {
    with_free_payload(raw, |payload| payload.dead_method())
}
