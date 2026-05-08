pub type EnvelopeResult = Result<Option<Vec<EnvelopeDto>>, EnvelopeError>;

pub struct EnvelopeDto {
    value: String,
}

impl EnvelopeDto {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dto:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-dto:{}", self.value)
    }
}

pub enum EnvelopeError {
    Empty,
}

pub fn selected_envelope(raw: &str) -> EnvelopeResult {
    if raw.trim().is_empty() {
        Err(EnvelopeError::Empty)
    } else {
        Ok(Some(vec![EnvelopeDto::new(raw)]))
    }
}

pub fn render_envelope(raw: &str) -> String {
    match selected_envelope(raw) {
        Ok(Some(items)) => items
            .into_iter()
            .map(|item| item.render())
            .collect::<Vec<_>>()
            .join(","),
        Ok(None) => "none".to_string(),
        Err(EnvelopeError::Empty) => "empty".to_string(),
    }
}

pub fn dead_live_envelope(raw: &str) -> String {
    EnvelopeDto::new(raw).dead_method()
}
