pub struct VecRetainMapPayload {
    value: String,
}

impl VecRetainMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-retain-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-retain-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-retain-map:{}", self.value)
    }
}

pub fn selected_vec_retain_map(raw: &str) -> String {
    let mut values = vec![
        VecRetainMapPayload::new(raw),
        VecRetainMapPayload::new("tail"),
    ];
    values.retain(|payload| payload.value.contains(raw));
    values
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vec_retain_map(raw: &str) -> String {
    VecRetainMapPayload::new(raw).unused_label()
}
