#[derive(Clone)]
pub struct SliceSwapIterPayload {
    value: String,
}

impl SliceSwapIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-swap-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("slice-swap-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-swap-iter:{}", self.value)
    }
}

pub fn selected_slice_swap_iter(raw: &str) -> String {
    let mut payloads = [
        SliceSwapIterPayload::new(raw),
        SliceSwapIterPayload::new("tail"),
    ];
    payloads.swap(0, 1);
    payloads
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "slice-swap-iter:missing".to_string())
}

pub fn dead_live_slice_swap_iter(raw: &str) -> String {
    SliceSwapIterPayload::new(raw).dead_method()
}
