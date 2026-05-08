pub struct StructInspectKey {
    value: String,
}

impl StructInspectKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn audit(&self, value: &StructInspectValue) -> String {
        format!("struct-inspect:{}:{}", self.value, value.render_label())
    }

    pub fn dead_method(&self) -> String {
        format!("dead-struct-inspect-key:{}", self.value)
    }
}

pub struct StructInspectValue {
    value: String,
}

impl StructInspectValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("struct-inspect-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-struct-inspect-value:{}", self.value)
    }
}

pub struct StructInspectPayload {
    key: StructInspectKey,
    value: StructInspectValue,
}

fn struct_inspect_payloads(raw: &str) -> Vec<StructInspectPayload> {
    raw.split(',')
        .map(|part| StructInspectPayload {
            key: StructInspectKey::new(part),
            value: StructInspectValue::new(part),
        })
        .collect()
}

pub fn selected_struct_inspect(raw: &str) -> String {
    let payloads = struct_inspect_payloads(raw);
    let mut audit = Vec::new();
    payloads
        .iter()
        .inspect(|StructInspectPayload { key, value }| audit.push(key.audit(value)))
        .map(|StructInspectPayload { value, .. }| value.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_struct_inspect(raw: &str) -> String {
    StructInspectKey::new(raw).dead_method()
}
