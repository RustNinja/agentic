#[derive(Clone)]
pub struct Regex {
    marker: String,
}

impl Regex {
    pub fn new(pattern: &str) -> Result<Self, RegexError> {
        if pattern.is_empty() {
            Err(RegexError {
                details: "empty".to_string(),
            })
        } else {
            Ok(Self {
                marker: pattern.to_string(),
            })
        }
    }

    pub fn captures(&self, raw: &str) -> Option<Captures> {
        raw.split_once(&self.marker).map(|(_, tail)| Captures {
            route_id: tail.trim_matches('/').to_string(),
        })
    }

    pub fn replace_all(&self, raw: &str, replacement: &str) -> String {
        raw.replace(&self.marker, replacement)
    }
}

pub struct Captures {
    route_id: String,
}

impl Captures {
    pub fn name(&self, name: &str) -> Option<Match> {
        (name == "id").then(|| Match {
            value: self.route_id.clone(),
        })
    }

    pub fn dead_summary(&self) -> String {
        format!("dead-captures:{}", self.route_id)
    }
}

pub struct Match {
    value: String,
}

impl Match {
    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn dead_value(&self) -> String {
        format!("dead-match:{}", self.value)
    }
}

pub struct RegexError {
    details: String,
}

impl RegexError {
    pub fn message(&self) -> &str {
        &self.details
    }

    pub fn dead_message(&self) -> String {
        format!("dead-regex-error:{}", self.details)
    }
}

pub fn dead_live_regex_report(raw: &str) -> String {
    Regex::new(raw)
        .map(|regex| regex.replace_all(raw, "dead-live"))
        .unwrap_or_else(|error| error.dead_message())
}
