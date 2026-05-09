pub struct VecSwapRemovePayload {
    value: String,
}

impl VecSwapRemovePayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-swap-remove:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-swap-remove:{}", self.value)
    }
}

fn vec_swap_remove_items(raw: &str) -> Vec<VecSwapRemovePayload> {
    vec![VecSwapRemovePayload::new(raw), VecSwapRemovePayload::new("tail")]
}

pub fn selected_vec_swap_remove(raw: &str) -> String {
    let mut payloads = vec_swap_remove_items(raw);
    payloads.swap_remove(0).render_label()
}

pub fn dead_live_vec_swap_remove(raw: &str) -> String {
    VecSwapRemovePayload::new(raw).dead_method()
}
