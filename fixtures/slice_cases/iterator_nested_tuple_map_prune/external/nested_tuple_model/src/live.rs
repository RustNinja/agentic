pub struct NestedTupleKey {
    value: String,
}

impl NestedTupleKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_nested(&self, value: &NestedTupleValue, meta: &NestedTupleMeta) -> String {
        format!(
            "nested-tuple:{}:{}:{}",
            self.value,
            value.render_label(),
            meta.render_tag()
        )
    }

    pub fn dead_method(&self) -> String {
        format!("dead-nested-tuple-key:{}", self.value)
    }
}

pub struct NestedTupleValue {
    value: String,
}

impl NestedTupleValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("nested-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-nested-tuple-value:{}", self.value)
    }
}

pub struct NestedTupleMeta {
    value: String,
}

impl NestedTupleMeta {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_tag(&self) -> String {
        format!("nested-meta:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-nested-tuple-meta:{}", self.value)
    }
}

fn nested_tuple_entries(
    raw: &str,
) -> Vec<((NestedTupleKey, NestedTupleValue), NestedTupleMeta)> {
    raw.split(',')
        .map(|part| {
            (
                (NestedTupleKey::new(part), NestedTupleValue::new(part)),
                NestedTupleMeta::new(part),
            )
        })
        .collect()
}

pub fn selected_nested_tuple_map(raw: &str) -> String {
    let entries = nested_tuple_entries(raw);
    entries
        .iter()
        .map(|((key, value), meta)| key.render_nested(value, meta))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_nested_tuple_map(raw: &str) -> String {
    NestedTupleKey::new(raw).dead_method()
}
