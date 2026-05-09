#[derive(Clone)]
pub struct VecSplitFirstMapPayload {
    value: String,
}

impl VecSplitFirstMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-split-first-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-split-first-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-split-first-map:{}", self.value)
    }
}

pub fn selected_vec_split_first_map(raw: &str) -> String {
    let payloads = vec![
        VecSplitFirstMapPayload::new(raw),
        VecSplitFirstMapPayload::new("tail"),
    ];
    payloads
        .split_first()
        .map(|(payload, _tail)| payload.render_label())
        .unwrap_or_else(|| "vec-split-first-map:missing".to_string())
}

pub fn dead_live_vec_split_first_map(raw: &str) -> String {
    VecSplitFirstMapPayload::new(raw).dead_method()
}
