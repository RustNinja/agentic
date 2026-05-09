pub struct OptionStructInner {
    value: String,
}

impl OptionStructInner {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-struct:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-struct:{}", self.value)
    }
}

pub struct OptionStructPayload {
    inner: OptionStructInner,
}

fn option_struct_payload(raw: &str) -> Option<OptionStructPayload> {
    if raw.trim().is_empty() {
        None
    } else {
        Some(OptionStructPayload {
            inner: OptionStructInner::new(raw),
        })
    }
}

pub fn selected_option_struct(raw: &str) -> String {
    match option_struct_payload(raw) {
        Some(OptionStructPayload { inner }) => inner.render_label(),
        None => "option-struct:missing".to_string(),
    }
}

pub fn dead_live_option_struct(raw: &str) -> String {
    OptionStructInner::new(raw).dead_method()
}
