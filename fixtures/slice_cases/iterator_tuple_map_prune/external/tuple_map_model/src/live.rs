pub struct TupleMapKey {
    value: String,
}

impl TupleMapKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_with(&self, value: &TupleMapValue) -> String {
        format!("tuple-map:{}:{}", self.value, value.render_label())
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-map-key:{}", self.value)
    }
}

pub struct TupleMapValue {
    value: String,
}

impl TupleMapValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-map-value:{}", self.value)
    }
}

fn tuple_map_entries(raw: &str) -> Vec<(TupleMapKey, TupleMapValue)> {
    raw.split(',')
        .map(|part| (TupleMapKey::new(part), TupleMapValue::new(part)))
        .collect()
}

pub fn selected_tuple_map(raw: &str) -> String {
    let entries = tuple_map_entries(raw);
    entries
        .iter()
        .map(|(key, value)| key.render_with(value))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_tuple_map(raw: &str) -> String {
    TupleMapKey::new(raw).dead_method()
}
