#[derive(Clone)]
pub struct VecTruncateIterPayload {
    value: String,
}

impl VecTruncateIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-truncate-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-truncate-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-truncate-iter:{}", self.value)
    }
}

pub fn selected_vec_truncate_iter(raw: &str) -> String {
    let mut payloads = vec![
        VecTruncateIterPayload::new(raw),
        VecTruncateIterPayload::new("tail"),
        VecTruncateIterPayload::new("extra"),
    ];
    payloads.truncate(2);
    payloads
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "vec-truncate-iter:missing".to_string())
}

pub fn dead_live_vec_truncate_iter(raw: &str) -> String {
    VecTruncateIterPayload::new(raw).dead_method()
}
