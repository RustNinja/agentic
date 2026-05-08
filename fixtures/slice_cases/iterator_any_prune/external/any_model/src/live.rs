pub struct AnyFlag {
    value: String,
}

impl AnyFlag {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn render(&self) -> String {
        format!("any:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-any:{}", self.value)
    }
}

fn build_flags(raw: &str) -> Vec<AnyFlag> {
    raw.split(',').map(AnyFlag::new).collect()
}

pub fn selected_any(raw: &str) -> String {
    if build_flags(raw).iter().any(|flag| flag.is_live()) {
        AnyFlag::new("live").render()
    } else {
        "any:none".to_string()
    }
}

pub fn dead_live_any(raw: &str) -> String {
    AnyFlag::new(raw).dead_method()
}
