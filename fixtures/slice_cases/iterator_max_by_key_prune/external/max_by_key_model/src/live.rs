pub struct MaxByKeyPayload {
    value: String,
}

impl MaxByKeyPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn rank(&self) -> usize {
        self.value.len()
    }

    pub fn render_label(&self) -> String {
        format!("max-by-key:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-max-by-key:{}", self.value)
    }
}

fn max_by_key_items(raw: &str) -> Vec<MaxByKeyPayload> {
    vec![MaxByKeyPayload::new(raw), MaxByKeyPayload::new("fallback")]
}

pub fn selected_max_by_key(raw: &str) -> String {
    max_by_key_items(raw)
        .into_iter()
        .max_by_key(|payload| payload.rank())
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "max-by-key:missing".to_string())
}

pub fn dead_live_max_by_key(raw: &str) -> String {
    MaxByKeyPayload::new(raw).dead_method()
}
