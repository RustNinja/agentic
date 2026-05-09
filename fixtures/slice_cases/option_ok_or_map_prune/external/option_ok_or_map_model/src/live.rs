#[derive(Clone)]
pub struct OptionOkOrMapPayload {
    value: String,
}

impl OptionOkOrMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-ok-or-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-ok-or-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-ok-or-map:{}", self.value)
    }
}

pub struct OptionOkOrMapError {
    value: String,
}

impl OptionOkOrMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("option-ok-or-map-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-ok-or-map-error:{}", self.value)
    }
}

pub fn selected_option_ok_or_map(raw: &str) -> String {
    let option = Some(OptionOkOrMapPayload::new(raw));
    option
        .ok_or(OptionOkOrMapError::new(raw))
        .map(|payload| payload.render_label())
        .unwrap_or_else(|err| err.render_error())
}

pub fn dead_live_option_ok_or_map(raw: &str) -> String {
    let mut payload = OptionOkOrMapPayload::new(raw);
    payload.bump_and_render()
}
