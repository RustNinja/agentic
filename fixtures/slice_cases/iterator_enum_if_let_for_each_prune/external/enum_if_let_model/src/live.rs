pub struct EnumIfLetKey {
    value: String,
}

impl EnumIfLetKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_with(&self, value: &EnumIfLetValue) -> String {
        format!("enum-if-let:{}:{}", self.value, value.render_label())
    }

    pub fn dead_method(&self) -> String {
        format!("dead-enum-if-let-key:{}", self.value)
    }
}

pub struct EnumIfLetValue {
    value: String,
}

impl EnumIfLetValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("enum-if-let-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-enum-if-let-value:{}", self.value)
    }
}

enum EnumIfLetEvent {
    Live {
        key: EnumIfLetKey,
        value: EnumIfLetValue,
    },
    Dead(DeadEnumIfLetPayload),
}

pub struct DeadEnumIfLetPayload {
    value: String,
}

fn enum_if_let_events(raw: &str) -> Vec<EnumIfLetEvent> {
    raw.split(',')
        .map(|part| EnumIfLetEvent::Live {
            key: EnumIfLetKey::new(part),
            value: EnumIfLetValue::new(part),
        })
        .collect()
}

pub fn selected_enum_if_let(raw: &str) -> String {
    let events = enum_if_let_events(raw);
    let mut rendered = Vec::new();
    events.iter().for_each(|event| {
        if let EnumIfLetEvent::Live { key, value } = event {
            rendered.push(key.render_with(value));
        }
    });
    rendered.join("|")
}

pub fn dead_live_enum_if_let(raw: &str) -> String {
    EnumIfLetKey::new(raw).dead_method()
}
