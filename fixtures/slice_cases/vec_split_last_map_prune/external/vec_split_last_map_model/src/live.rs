#[derive(Clone)]
pub struct VecSplitLastMapPayload {
    value: String,
}

impl VecSplitLastMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-split-last-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-split-last-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-split-last-map:{}", self.value)
    }
}

pub fn selected_vec_split_last_map(raw: &str) -> String {
    let payloads = vec![
        VecSplitLastMapPayload::new("head"),
        VecSplitLastMapPayload::new(raw),
    ];
    payloads
        .split_last()
        .map(|(payload, _tail)| payload.render_label())
        .unwrap_or_else(|| "vec-split-last-map:missing".to_string())
}

pub fn dead_live_vec_split_last_map(raw: &str) -> String {
    VecSplitLastMapPayload::new(raw).dead_method()
}
