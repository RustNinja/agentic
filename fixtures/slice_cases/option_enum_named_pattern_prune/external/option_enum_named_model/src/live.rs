pub struct OptionEnumNamedPayload {
    value: String,
}

impl OptionEnumNamedPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-enum-named:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-enum-named:{}", self.value)
    }
}

pub enum OptionEnumNamedEvent {
    Live { payload: OptionEnumNamedPayload },
    Empty,
}

fn option_enum_named_payload(raw: &str) -> Option<OptionEnumNamedEvent> {
    if raw.trim().is_empty() {
        Some(OptionEnumNamedEvent::Empty)
    } else {
        Some(OptionEnumNamedEvent::Live {
            payload: OptionEnumNamedPayload::new(raw),
        })
    }
}

pub fn selected_option_enum_named(raw: &str) -> String {
    match option_enum_named_payload(raw) {
        Some(OptionEnumNamedEvent::Live { payload }) => payload.render_label(),
        Some(OptionEnumNamedEvent::Empty) | None => "option-enum-named:missing".to_string(),
    }
}

pub fn dead_live_option_enum_named(raw: &str) -> String {
    OptionEnumNamedPayload::new(raw).dead_method()
}
