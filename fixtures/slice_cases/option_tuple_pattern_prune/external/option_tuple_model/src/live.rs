pub struct OptionTupleKey {
    value: String,
}

impl OptionTupleKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_with(&self, value: OptionTupleValue) -> String {
        format!("option-tuple:{}:{}", self.value, value.render_label())
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-tuple:{}", self.value)
    }
}

pub struct OptionTupleValue {
    value: String,
}

impl OptionTupleValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        self.value.clone()
    }
}

fn option_tuple_payload(raw: &str) -> Option<(OptionTupleKey, OptionTupleValue)> {
    if raw.trim().is_empty() {
        None
    } else {
        Some((OptionTupleKey::new(raw), OptionTupleValue::new(raw)))
    }
}

pub fn selected_option_tuple(raw: &str) -> String {
    match option_tuple_payload(raw) {
        Some((key, value)) => key.render_with(value),
        None => "option-tuple:missing".to_string(),
    }
}

pub fn dead_live_option_tuple(raw: &str) -> String {
    OptionTupleKey::new(raw).dead_method()
}
