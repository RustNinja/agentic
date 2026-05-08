use std::str::FromStr;

pub struct ParseRecord {
    label: String,
}

impl ParseRecord {
    pub fn render(&self) -> String {
        format!("parse:{}", self.label)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-parse:{}", self.label)
    }
}

pub struct ParseError {
    message: String,
}

impl ParseError {
    pub fn render(&self) -> String {
        format!("parse-error:{}", self.message)
    }
}

impl FromStr for ParseRecord {
    type Err = ParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
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
}

pub fn selected_parse(raw: &str) -> String {
    match raw.parse::<ParseRecord>() {
        Ok(record) => record.render(),
        Err(err) => err.render(),
    }
}

pub fn dead_live_parse(raw: &str) -> String {
    raw.parse::<ParseRecord>()
        .map(|record| record.dead_method())
        .unwrap_or_else(|_| "dead-parse-fallback".to_string())
}
