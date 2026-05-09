pub struct MinByPayload {
    value: String,
}

impl MinByPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn rank(&self) -> usize {
        self.value.len()
    }

    pub fn render_label(&self) -> String {
        format!("min-by:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-min-by:{}", self.value)
    }
}

fn min_by_items(raw: &str) -> Vec<MinByPayload> {
    vec![MinByPayload::new(raw), MinByPayload::new("fallback")]
}

pub fn selected_min_by(raw: &str) -> String {
    min_by_items(raw)
        .into_iter()
        .min_by(|left, right| left.rank().cmp(&right.rank()))
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "min-by:missing".to_string())
}

pub fn dead_live_min_by(raw: &str) -> String {
    MinByPayload::new(raw).dead_method()
}
