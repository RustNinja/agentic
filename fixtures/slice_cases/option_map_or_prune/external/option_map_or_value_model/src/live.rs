pub struct OptionMapOrValuePayload {
    value: String,
}

impl OptionMapOrValuePayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-map-or-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-map-or-value:{}", self.value)
    }
}

pub struct OptionMapOrValueDefault {
    value: String,
}

impl OptionMapOrValueDefault {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render_missing(&self) -> String {
        format!("option-map-or-value-missing:{}", self.value)
    }
}

fn option_map_or_value_payload(raw: &str) -> Option<OptionMapOrValuePayload> {
    (!raw.trim().is_empty()).then(|| OptionMapOrValuePayload::new(raw))
}

pub fn selected_option_map_or_value(raw: &str) -> String {
    option_map_or_value_payload(raw).map_or(
        OptionMapOrValueDefault::new(raw).render_missing(),
        |payload| payload.render_label(),
    )
}

pub fn dead_live_option_map_or_value(raw: &str) -> String {
    OptionMapOrValuePayload::new(raw).dead_method()
}
