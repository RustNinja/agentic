pub struct OptionMapOrPayload {
    value: String,
}

impl OptionMapOrPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-map-or:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-map-or:{}", self.value)
    }
}

pub struct OptionMapOrFallback {
    value: String,
}

impl OptionMapOrFallback {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render_missing(&self) -> String {
        format!("option-map-or-missing:{}", self.value)
    }
}

fn option_map_or_payload(raw: &str) -> Option<OptionMapOrPayload> {
    (!raw.trim().is_empty()).then(|| OptionMapOrPayload::new(raw))
}

pub fn selected_option_map_or(raw: &str) -> String {
    option_map_or_payload(raw).map_or_else(
        || OptionMapOrFallback::new(raw).render_missing(),
        |payload| payload.render_label(),
    )
}

pub fn dead_live_option_map_or(raw: &str) -> String {
    OptionMapOrPayload::new(raw).dead_method()
}
