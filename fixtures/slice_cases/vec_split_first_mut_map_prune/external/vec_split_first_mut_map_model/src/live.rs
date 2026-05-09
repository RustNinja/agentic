#[derive(Clone)]
pub struct VecSplitFirstMutMapPayload {
    value: String,
}

impl VecSplitFirstMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-split-first-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-split-first-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-split-first-mut-map:{}", self.value)
    }
}

pub fn selected_vec_split_first_mut_map(raw: &str) -> String {
    let mut payloads = vec![
        VecSplitFirstMutMapPayload::new(raw),
        VecSplitFirstMutMapPayload::new("tail"),
    ];
    payloads
        .split_first_mut()
        .map(|(payload, _tail)| payload.bump_and_render())
        .unwrap_or_else(|| "vec-split-first-mut-map:missing".to_string())
}

pub fn dead_live_vec_split_first_mut_map(raw: &str) -> String {
    VecSplitFirstMutMapPayload::new(raw).dead_method()
}
