pub struct TupleFilterKey {
    value: String,
}

impl TupleFilterKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn accepts(&self, value: &TupleFilterValue) -> bool {
        value.render_label().contains(&self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-filter-key:{}", self.value)
    }
}

pub struct TupleFilterValue {
    value: String,
}

impl TupleFilterValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("filter-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-filter-value:{}", self.value)
    }
}

fn tuple_filter_entries(raw: &str) -> Vec<(TupleFilterKey, TupleFilterValue)> {
    raw.split(',')
        .map(|part| (TupleFilterKey::new(part), TupleFilterValue::new(part)))
        .collect()
}

pub fn selected_tuple_filter(raw: &str) -> String {
    let entries = tuple_filter_entries(raw);
    entries
        .iter()
        .filter(|(key, value)| key.accepts(value))
        .map(|(_, value)| value.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_tuple_filter(raw: &str) -> String {
    TupleFilterKey::new(raw).dead_method()
}
