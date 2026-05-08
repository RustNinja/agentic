pub struct StructFilterKey {
    value: String,
}

impl StructFilterKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn accepts(&self, value: &StructFilterValue) -> bool {
        value.render_label().contains(&self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-struct-filter-key:{}", self.value)
    }
}

pub struct StructFilterValue {
    value: String,
}

impl StructFilterValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("struct-filter-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-struct-filter-value:{}", self.value)
    }
}

pub struct StructFilterPayload {
    key: StructFilterKey,
    value: StructFilterValue,
}

fn struct_filter_payloads(raw: &str) -> Vec<StructFilterPayload> {
    raw.split(',')
        .map(|part| StructFilterPayload {
            key: StructFilterKey::new(part),
            value: StructFilterValue::new(part),
        })
        .collect()
}

pub fn selected_struct_filter(raw: &str) -> String {
    let payloads = struct_filter_payloads(raw);
    payloads
        .iter()
        .filter(|StructFilterPayload { key, value }| key.accepts(value))
        .map(|StructFilterPayload { value, .. }| value.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_struct_filter(raw: &str) -> String {
    StructFilterKey::new(raw).dead_method()
}
