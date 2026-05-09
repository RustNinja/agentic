use std::mem;

#[derive(Clone)]
pub struct MemSwapMapPayload {
    value: String,
}

impl MemSwapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mem-swap-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("mem-swap-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mem-swap-map:{}", self.value)
    }
}

pub fn selected_mem_swap_map(raw: &str) -> String {
    let mut left = MemSwapMapPayload::new(raw);
    let mut right = MemSwapMapPayload::new("swap");
    mem::swap(&mut left, &mut right);
    left.render_label()
}

pub fn dead_live_mem_swap_map(raw: &str) -> String {
    MemSwapMapPayload::new(raw).dead_method()
}
