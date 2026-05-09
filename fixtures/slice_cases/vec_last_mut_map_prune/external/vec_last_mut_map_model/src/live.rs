pub struct VecLastMutMapPayload {
    value: String,
}

impl VecLastMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-last-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-last-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-last-mut-map:{}", self.value)
    }
}

pub fn selected_vec_last_mut_map(raw: &str) -> String {
    let mut values = vec![
        VecLastMutMapPayload::new("head"),
        VecLastMutMapPayload::new(raw),
    ];
    values
        .last_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vec_last_mut_map(raw: &str) -> String {
    VecLastMutMapPayload::new(raw).unused_label()
}
