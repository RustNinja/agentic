pub struct MaxByPayload {
    value: String,
}

impl MaxByPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn rank(&self) -> usize {
        self.value.len()
    }

    pub fn render_label(&self) -> String {
        format!("max-by:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-max-by:{}", self.value)
    }
}

fn max_by_items(raw: &str) -> Vec<MaxByPayload> {
    vec![MaxByPayload::new(raw), MaxByPayload::new("fallback")]
}

pub fn selected_max_by(raw: &str) -> String {
    max_by_items(raw)
        .into_iter()
        .max_by(|left, right| left.rank().cmp(&right.rank()))
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "max-by:missing".to_string())
}

pub fn dead_live_max_by(raw: &str) -> String {
    MaxByPayload::new(raw).dead_method()
}
