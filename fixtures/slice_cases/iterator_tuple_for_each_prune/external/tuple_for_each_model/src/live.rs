pub struct TupleForEachKey {
    value: String,
}

impl TupleForEachKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_with(&self, value: &TupleForEachValue) -> String {
        format!("tuple-for-each:{}:{}", self.value, value.render_label())
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-for-each-key:{}", self.value)
    }
}

pub struct TupleForEachValue {
    value: String,
}

impl TupleForEachValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for-each-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-for-each-value:{}", self.value)
    }
}

fn tuple_for_each_entries(raw: &str) -> Vec<(TupleForEachKey, TupleForEachValue)> {
    raw.split(',')
        .map(|part| (TupleForEachKey::new(part), TupleForEachValue::new(part)))
        .collect()
}

pub fn selected_tuple_for_each(raw: &str) -> String {
    let entries = tuple_for_each_entries(raw);
    let mut rendered = Vec::new();
    entries
        .iter()
        .for_each(|(key, value)| rendered.push(key.render_with(value)));
    rendered.join("|")
}

pub fn dead_live_tuple_for_each(raw: &str) -> String {
    TupleForEachKey::new(raw).dead_method()
}
