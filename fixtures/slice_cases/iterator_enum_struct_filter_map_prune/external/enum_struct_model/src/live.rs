pub struct EnumStructKey {
    value: String,
}

impl EnumStructKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_with(&self, value: &EnumStructValue) -> String {
        format!("enum-struct:{}:{}", self.value, value.render_label())
    }

    pub fn dead_method(&self) -> String {
        format!("dead-enum-struct-key:{}", self.value)
    }
}

pub struct EnumStructValue {
    value: String,
}

impl EnumStructValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("enum-struct-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-enum-struct-value:{}", self.value)
    }
}

enum EnumStructEvent {
    Live {
        key: EnumStructKey,
        value: EnumStructValue,
    },
    Dead(DeadEnumStructPayload),
}

pub struct DeadEnumStructPayload {
    value: String,
}

fn enum_struct_events(raw: &str) -> Vec<EnumStructEvent> {
    raw.split(',')
        .map(|part| EnumStructEvent::Live {
            key: EnumStructKey::new(part),
            value: EnumStructValue::new(part),
        })
        .collect()
}

pub fn selected_enum_struct(raw: &str) -> String {
    let events = enum_struct_events(raw);
    events
        .iter()
        .filter_map(|event| match event {
            EnumStructEvent::Live { key, value } => Some(key.render_with(value)),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_enum_struct(raw: &str) -> String {
    EnumStructKey::new(raw).dead_method()
}
