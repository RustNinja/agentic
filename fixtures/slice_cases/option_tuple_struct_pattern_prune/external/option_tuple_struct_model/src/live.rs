pub struct OptionTupleStructInner {
    value: String,
}

impl OptionTupleStructInner {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-tuple-struct:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-tuple-struct:{}", self.value)
    }
}

pub struct OptionTupleStructPayload(pub OptionTupleStructInner);

fn option_tuple_struct_payload(raw: &str) -> Option<OptionTupleStructPayload> {
    if raw.trim().is_empty() {
        None
    } else {
        Some(OptionTupleStructPayload(OptionTupleStructInner::new(raw)))
    }
}

pub fn selected_option_tuple_struct(raw: &str) -> String {
    match option_tuple_struct_payload(raw) {
        Some(OptionTupleStructPayload(inner)) => inner.render_label(),
        None => "option-tuple-struct:missing".to_string(),
    }
}

pub fn dead_live_option_tuple_struct(raw: &str) -> String {
    OptionTupleStructInner::new(raw).dead_method()
}
