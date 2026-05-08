pub struct EnumMatchKey {
    value: String,
}

impl EnumMatchKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_with(&self, value: &EnumMatchValue) -> String {
        format!("enum-match:{}:{}", self.value, value.render_label())
    }

    pub fn dead_method(&self) -> String {
        format!("dead-enum-match-key:{}", self.value)
    }
}

pub struct EnumMatchValue {
    value: String,
}

impl EnumMatchValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("enum-match-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-enum-match-value:{}", self.value)
    }
}

enum EnumMatchEvent {
    Live(EnumMatchKey, EnumMatchValue),
    Dead(DeadEnumMatchPayload),
}

pub struct DeadEnumMatchPayload {
    value: String,
}

fn enum_match_events(raw: &str) -> Vec<EnumMatchEvent> {
    raw.split(',')
        .map(|part| EnumMatchEvent::Live(EnumMatchKey::new(part), EnumMatchValue::new(part)))
        .collect()
}

pub fn selected_enum_match(raw: &str) -> String {
    let events = enum_match_events(raw);
    events
        .iter()
        .map(|event| match event {
            EnumMatchEvent::Live(key, value) => key.render_with(value),
            _ => String::new(),
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_enum_match(raw: &str) -> String {
    EnumMatchKey::new(raw).dead_method()
}
