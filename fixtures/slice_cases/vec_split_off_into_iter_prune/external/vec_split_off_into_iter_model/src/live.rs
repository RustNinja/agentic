#[derive(Clone)]
pub struct VecSplitOffIntoIterPayload {
    value: String,
}

impl VecSplitOffIntoIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-split-off-into-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-split-off-into-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-split-off-into-iter:{}", self.value)
    }
}

pub fn selected_vec_split_off_into_iter(raw: &str) -> String {
    let mut payloads = vec![
        VecSplitOffIntoIterPayload::new(raw),
        VecSplitOffIntoIterPayload::new("tail"),
    ];
    payloads
        .split_off(1)
        .into_iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "vec-split-off-into-iter:missing".to_string())
}

pub fn dead_live_vec_split_off_into_iter(raw: &str) -> String {
    VecSplitOffIntoIterPayload::new(raw).dead_method()
}
