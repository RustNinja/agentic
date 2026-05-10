pub struct OptionUnwrapUncheckedMapPayload {
    value: String,
}

impl OptionUnwrapUncheckedMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-unwrap-unchecked-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("option-unwrap-unchecked-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-option-unwrap-unchecked-map:{}", self.value)
    }
}

pub fn selected_option_unwrap_unchecked_map(raw: &str) -> String {
    let value = Some(OptionUnwrapUncheckedMapPayload::new(raw));
    unsafe { value.unwrap_unchecked().render_label() }
}

pub fn dead_live_option_unwrap_unchecked_map(raw: &str) -> String {
    OptionUnwrapUncheckedMapPayload::new(raw).unused_label()
}
