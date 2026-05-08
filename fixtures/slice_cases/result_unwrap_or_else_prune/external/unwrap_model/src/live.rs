pub struct UnwrapValue {
    value: String,
}

impl UnwrapValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("unwrap:{}", self.value)
    }
}

pub struct UnwrapError {
    raw: String,
}

impl UnwrapError {
    pub fn new(raw: &str) -> Self {
        Self {
            raw: raw.to_string(),
        }
    }

    pub fn recover(self) -> UnwrapValue {
        UnwrapValue::new(&format!("fallback-{}", self.raw))
    }

    pub fn dead_method(self) -> String {
        format!("dead-error:{}", self.raw)
    }
}

fn parse_unwrap(raw: &str) -> Result<UnwrapValue, UnwrapError> {
    if raw.trim().is_empty() {
        Err(UnwrapError::new(raw))
    } else {
        Ok(UnwrapValue::new(raw))
    }
}

pub fn selected_unwrap(raw: &str) -> String {
    parse_unwrap(raw)
        .unwrap_or_else(|err| err.recover())
        .render()
}

pub fn dead_live_unwrap(raw: &str) -> String {
    UnwrapError::new(raw).dead_method()
}
