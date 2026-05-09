#[derive(Clone)]
pub struct VecSplitLastMutMapPayload {
    value: String,
}

impl VecSplitLastMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-split-last-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-split-last-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-split-last-mut-map:{}", self.value)
    }
}

pub fn selected_vec_split_last_mut_map(raw: &str) -> String {
    let mut payloads = vec![
        VecSplitLastMutMapPayload::new("head"),
        VecSplitLastMutMapPayload::new(raw),
    ];
    payloads
        .split_last_mut()
        .map(|(payload, _tail)| payload.bump_and_render())
        .unwrap_or_else(|| "vec-split-last-mut-map:missing".to_string())
}

pub fn dead_live_vec_split_last_mut_map(raw: &str) -> String {
    VecSplitLastMutMapPayload::new(raw).dead_method()
}
