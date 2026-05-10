pub struct VecSwapGetMapPayload {
    value: String,
}

impl VecSwapGetMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-swap-get-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-swap-get-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-swap-get-map:{}", self.value)
    }
}

pub fn selected_vec_swap_get_map(raw: &str) -> String {
    let mut values = vec![
        VecSwapGetMapPayload::new("left"),
        VecSwapGetMapPayload::new(raw),
    ];
    values.swap(0, 1);
    values
        .get(0)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vec_swap_get_map(raw: &str) -> String {
    VecSwapGetMapPayload::new(raw).unused_label()
}
