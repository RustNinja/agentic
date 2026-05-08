pub enum WireError {
    Empty,
    Parse(ParseFailure),
}

impl WireError {
    pub fn message(&self) -> String {
        match self {
            WireError::Empty => "empty".to_string(),
            WireError::Parse(source) => source.describe(),
        }
    }

    pub fn dead_method(&self) -> String {
        "dead-error".to_string()
    }
}

pub struct ParseFailure {
    input: String,
}

impl ParseFailure {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
        }
    }

    pub fn describe(&self) -> String {
        format!("parse:{}", self.input)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-parse:{}", self.input)
    }
}

impl From<ParseFailure> for WireError {
    fn from(value: ParseFailure) -> Self {
        WireError::Parse(value)
    }
}

fn normalize(raw: &str) -> Result<String, ParseFailure> {
    let value = raw.trim();
    if value == "bad" {
        Err(ParseFailure::new(raw))
    } else {
        Ok(value.to_string())
    }
}

pub fn parse_wire(raw: &str) -> Result<String, WireError> {
    if raw.trim().is_empty() {
        return Err(WireError::Empty);
    }
    let value = normalize(raw)?;
    Ok(format!("wire:{value}"))
}

pub fn selected_error(raw: &str) -> String {
    parse_wire(raw).unwrap_or_else(|error| error.message())
}

pub fn dead_live_error(raw: &str) -> String {
    ParseFailure::new(raw).dead_method()
}
