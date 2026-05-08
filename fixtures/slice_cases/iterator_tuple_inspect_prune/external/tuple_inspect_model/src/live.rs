pub struct TupleInspectKey {
    value: String,
}

impl TupleInspectKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn audit(&self, value: &TupleInspectValue) -> String {
        format!("tuple-inspect:{}:{}", self.value, value.render_label())
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-inspect-key:{}", self.value)
    }
}

pub struct TupleInspectValue {
    value: String,
}

impl TupleInspectValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("inspect-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-inspect-value:{}", self.value)
    }
}

fn tuple_inspect_entries(raw: &str) -> Vec<(TupleInspectKey, TupleInspectValue)> {
    raw.split(',')
        .map(|part| (TupleInspectKey::new(part), TupleInspectValue::new(part)))
        .collect()
}

pub fn selected_tuple_inspect(raw: &str) -> String {
    let entries = tuple_inspect_entries(raw);
    let mut audit = Vec::new();
    let rendered = entries
        .iter()
        .inspect(|(key, value)| audit.push(key.audit(value)))
        .map(|(_, value)| value.render_label())
        .collect::<Vec<_>>();
    format!("{}:{}", audit.len(), rendered.join("|"))
}

pub fn dead_live_tuple_inspect(raw: &str) -> String {
    TupleInspectKey::new(raw).dead_method()
}
