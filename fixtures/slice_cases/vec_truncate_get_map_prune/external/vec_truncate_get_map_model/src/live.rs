pub struct VecTruncateGetMapPayload {
    value: String,
}

impl VecTruncateGetMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-truncate-get-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-truncate-get-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-truncate-get-map:{}", self.value)
    }
}

pub fn selected_vec_truncate_get_map(raw: &str) -> String {
    let mut values = vec![
        VecTruncateGetMapPayload::new(raw),
        VecTruncateGetMapPayload::new("tail"),
    ];
    values.truncate(1);
    values
        .get(0)
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_vec_truncate_get_map(raw: &str) -> String {
    VecTruncateGetMapPayload::new(raw).unused_label()
}
