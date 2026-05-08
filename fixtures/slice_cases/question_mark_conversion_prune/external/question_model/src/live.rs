pub type QuestionResult<T> = Result<T, WireError>;

pub struct ParsedQuestion {
    label: String,
}

impl ParsedQuestion {
    pub fn parse(raw: &str) -> Result<Self, ParseError> {
        let label = raw.trim();
        if label.is_empty() {
            Err(ParseError {
                message: "empty".to_string(),
            })
        } else {
            Ok(Self {
                label: label.to_string(),
            })
        }
    }

    pub fn render(&self) -> String {
        format!("question:{}", self.label)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-question:{}", self.label)
    }
}

pub struct ParseError {
    message: String,
}

pub struct WireError {
    message: String,
}

impl WireError {
    pub fn render(&self) -> String {
        format!("wire-error:{}", self.message)
    }
}

impl From<ParseError> for WireError {
    fn from(value: ParseError) -> Self {
        Self {
            message: value.message,
        }
    }
}

pub fn selected_question(raw: &str) -> QuestionResult<String> {
    let parsed = ParsedQuestion::parse(raw)?;
    Ok(parsed.render())
}

pub fn dead_live_question(raw: &str) -> String {
    ParsedQuestion::parse(raw)
        .map(|value| value.dead_method())
        .unwrap_or_else(|_| "dead-question-fallback".to_string())
}
