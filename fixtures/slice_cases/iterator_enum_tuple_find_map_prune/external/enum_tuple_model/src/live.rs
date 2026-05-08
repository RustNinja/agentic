pub struct EnumTupleKey {
    value: String,
}

impl EnumTupleKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn maybe_render(&self, value: &EnumTupleValue) -> Option<String> {
        Some(format!(
            "enum-tuple:{}:{}",
            self.value,
            value.render_label()
        ))
    }

    pub fn dead_method(&self) -> String {
        format!("dead-enum-tuple-key:{}", self.value)
    }
}

pub struct EnumTupleValue {
    value: String,
}

impl EnumTupleValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("enum-tuple-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-enum-tuple-value:{}", self.value)
    }
}

enum EnumTupleEvent {
    Live(EnumTupleKey, EnumTupleValue),
    Dead(DeadEnumTuplePayload),
}

pub struct DeadEnumTuplePayload {
    value: String,
}

fn enum_tuple_events(raw: &str) -> Vec<EnumTupleEvent> {
    raw.split(',')
        .map(|part| EnumTupleEvent::Live(EnumTupleKey::new(part), EnumTupleValue::new(part)))
        .collect()
}

pub fn selected_enum_tuple(raw: &str) -> String {
    let events = enum_tuple_events(raw);
    events
        .iter()
        .find_map(|event| match event {
            EnumTupleEvent::Live(key, value) => key.maybe_render(value),
            _ => None,
        })
        .unwrap_or_else(|| "enum-tuple:missing".to_string())
}

pub fn dead_live_enum_tuple(raw: &str) -> String {
    EnumTupleKey::new(raw).dead_method()
}
