pub struct OptionExpectPayload {
    value: String,
}

impl OptionExpectPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn render_label(&self) -> String {
        format!("option-expect:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-expect:{}", self.value)
    }
}

fn option_expect_payload(raw: &str) -> Option<OptionExpectPayload> {
    Some(OptionExpectPayload::new(raw))
}

pub fn selected_option_expect(raw: &str) -> String {
    option_expect_payload(raw)
        .expect("fixture payload should exist")
        .render_label()
}

pub fn dead_live_option_expect(raw: &str) -> String {
    OptionExpectPayload::new(raw).dead_method()
}
