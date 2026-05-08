pub struct OptionPayload {
    label: String,
}

impl OptionPayload {
    pub fn render(&self) -> String {
        format!("option:{}", self.label)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option:{}", self.label)
    }
}

pub struct OptionHolder {
    payload: Option<OptionPayload>,
}

impl OptionHolder {
    pub fn new(raw: &str) -> Self {
        Self {
            payload: Some(OptionPayload {
                label: raw.trim().to_string(),
            }),
        }
    }

    pub fn dead_holder_method(&self) -> String {
        self.payload
            .as_ref()
            .map(OptionPayload::dead_method)
            .unwrap_or_default()
    }
}

pub fn selected_option(raw: &str) -> String {
    let holder = OptionHolder::new(raw);
    if let Some(payload) = holder.payload.as_ref() {
        payload.render()
    } else {
        "option:empty".to_string()
    }
}

pub fn dead_live_option(raw: &str) -> String {
    OptionHolder::new(raw).dead_holder_method()
}
