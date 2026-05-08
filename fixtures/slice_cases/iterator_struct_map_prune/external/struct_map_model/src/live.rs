pub struct StructMapKey {
    value: String,
}

impl StructMapKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_with(&self, value: &StructMapValue) -> String {
        format!("struct-map:{}:{}", self.value, value.render_label())
    }

    pub fn dead_method(&self) -> String {
        format!("dead-struct-map-key:{}", self.value)
    }
}

pub struct StructMapValue {
    value: String,
}

impl StructMapValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("struct-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-struct-map-value:{}", self.value)
    }
}

pub struct StructMapPayload {
    key: StructMapKey,
    value: StructMapValue,
}

fn struct_map_payloads(raw: &str) -> Vec<StructMapPayload> {
    raw.split(',')
        .map(|part| StructMapPayload {
            key: StructMapKey::new(part),
            value: StructMapValue::new(part),
        })
        .collect()
}

pub fn selected_struct_map(raw: &str) -> String {
    let payloads = struct_map_payloads(raw);
    payloads
        .iter()
        .map(|StructMapPayload { key, value }| key.render_with(value))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_struct_map(raw: &str) -> String {
    StructMapKey::new(raw).dead_method()
}
