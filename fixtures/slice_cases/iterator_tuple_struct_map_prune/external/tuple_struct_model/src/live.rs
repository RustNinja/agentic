pub struct TupleStructKey {
    value: String,
}

impl TupleStructKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_with(&self, value: &TupleStructValue) -> String {
        format!("tuple-struct:{}:{}", self.value, value.render_label())
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-struct-key:{}", self.value)
    }
}

pub struct TupleStructValue {
    value: String,
}

impl TupleStructValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("tuple-struct-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-struct-value:{}", self.value)
    }
}

pub struct TupleStructPayload(TupleStructKey, TupleStructValue);

fn tuple_struct_payloads(raw: &str) -> Vec<TupleStructPayload> {
    raw.split(',')
        .map(|part| TupleStructPayload(TupleStructKey::new(part), TupleStructValue::new(part)))
        .collect()
}

pub fn selected_tuple_struct(raw: &str) -> String {
    let payloads = tuple_struct_payloads(raw);
    payloads
        .iter()
        .map(|TupleStructPayload(key, value)| key.render_with(value))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_tuple_struct(raw: &str) -> String {
    TupleStructKey::new(raw).dead_method()
}
