pub struct EnumFlatMapKey {
    value: String,
}

impl EnumFlatMapKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn expand_with(&self, value: &EnumFlatMapValue) -> Vec<String> {
        vec![format!(
            "enum-flat-map:{}:{}",
            self.value,
            value.render_label()
        )]
    }

    pub fn dead_method(&self) -> String {
        format!("dead-enum-flat-map-key:{}", self.value)
    }
}

pub struct EnumFlatMapValue {
    value: String,
}

impl EnumFlatMapValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("enum-flat-map-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-enum-flat-map-value:{}", self.value)
    }
}

enum EnumFlatMapEvent {
    Live {
        key: EnumFlatMapKey,
        value: EnumFlatMapValue,
    },
    Dead(DeadEnumFlatMapPayload),
}

pub struct DeadEnumFlatMapPayload {
    value: String,
}

fn enum_flat_map_events(raw: &str) -> Vec<EnumFlatMapEvent> {
    raw.split(',')
        .map(|part| EnumFlatMapEvent::Live {
            key: EnumFlatMapKey::new(part),
            value: EnumFlatMapValue::new(part),
        })
        .collect()
}

pub fn selected_enum_flat_map(raw: &str) -> String {
    let events = enum_flat_map_events(raw);
    events
        .iter()
        .flat_map(|event| match event {
            EnumFlatMapEvent::Live { key, value } => key.expand_with(value),
            _ => Vec::new(),
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_enum_flat_map(raw: &str) -> String {
    EnumFlatMapKey::new(raw).dead_method()
}
