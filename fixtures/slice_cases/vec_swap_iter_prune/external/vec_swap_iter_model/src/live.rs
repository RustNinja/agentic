#[derive(Clone)]
pub struct VecSwapIterPayload {
    value: String,
}

impl VecSwapIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-swap-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-swap-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-swap-iter:{}", self.value)
    }
}

pub fn selected_vec_swap_iter(raw: &str) -> String {
    let mut payloads = vec![
        VecSwapIterPayload::new(raw),
        VecSwapIterPayload::new("tail"),
    ];
    payloads.swap(0, 1);
    payloads
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "vec-swap-iter:missing".to_string())
}

pub fn dead_live_vec_swap_iter(raw: &str) -> String {
    VecSwapIterPayload::new(raw).dead_method()
}
