#[derive(Clone)]
pub struct ArrayIntoIterSkipMapPayload {
    value: String,
}

impl ArrayIntoIterSkipMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("array-into-iter-skip-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("array-into-iter-skip-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-array-into-iter-skip-map:{}", self.value)
    }
}

pub fn selected_array_into_iter_skip_map(raw: &str) -> String {
    let items = [
        ArrayIntoIterSkipMapPayload::new("skip"),
        ArrayIntoIterSkipMapPayload::new(raw),
    ];
    items
        .into_iter()
        .skip(1)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| format!("array-into-iter-skip-map:missing"))
}

pub fn dead_live_array_into_iter_skip_map(raw: &str) -> String {
    let mut payload = ArrayIntoIterSkipMapPayload::new(raw);
    payload.bump_and_render()
}
