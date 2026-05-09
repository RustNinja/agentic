pub struct MinByKeyPayload {
    value: String,
}

impl MinByKeyPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn rank(&self) -> usize {
        self.value.len()
    }

    pub fn render_label(&self) -> String {
        format!("min-by-key:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-min-by-key:{}", self.value)
    }
}

fn min_by_key_items(raw: &str) -> Vec<MinByKeyPayload> {
    vec![MinByKeyPayload::new(raw), MinByKeyPayload::new("fallback")]
}

pub fn selected_min_by_key(raw: &str) -> String {
    min_by_key_items(raw)
        .into_iter()
        .min_by_key(|payload| payload.rank())
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "min-by-key:missing".to_string())
}

pub fn dead_live_min_by_key(raw: &str) -> String {
    MinByKeyPayload::new(raw).dead_method()
}
