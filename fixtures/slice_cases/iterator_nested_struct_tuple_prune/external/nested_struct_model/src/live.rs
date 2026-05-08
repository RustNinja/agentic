pub struct NestedStructKey {
    value: String,
}

impl NestedStructKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_with(&self, value: &NestedStructValue, meta: &NestedStructMeta) -> String {
        format!(
            "nested-struct:{}:{}:{}",
            self.value,
            value.render_label(),
            meta.render_tag()
        )
    }

    pub fn dead_method(&self) -> String {
        format!("dead-nested-struct-key:{}", self.value)
    }
}

pub struct NestedStructValue {
    value: String,
}

impl NestedStructValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("nested-struct-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-nested-struct-value:{}", self.value)
    }
}

pub struct NestedStructMeta {
    value: String,
}

impl NestedStructMeta {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_tag(&self) -> String {
        format!("nested-struct-meta:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-nested-struct-meta:{}", self.value)
    }
}

pub struct NestedStructPayload {
    key: NestedStructKey,
    value: NestedStructValue,
}

fn nested_struct_payloads(raw: &str) -> Vec<(NestedStructPayload, NestedStructMeta)> {
    raw.split(',')
        .map(|part| {
            (
                NestedStructPayload {
                    key: NestedStructKey::new(part),
                    value: NestedStructValue::new(part),
                },
                NestedStructMeta::new(part),
            )
        })
        .collect()
}

pub fn selected_nested_struct(raw: &str) -> String {
    let payloads = nested_struct_payloads(raw);
    payloads
        .iter()
        .map(|(NestedStructPayload { key, value }, meta)| key.render_with(value, meta))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_nested_struct(raw: &str) -> String {
    NestedStructKey::new(raw).dead_method()
}
