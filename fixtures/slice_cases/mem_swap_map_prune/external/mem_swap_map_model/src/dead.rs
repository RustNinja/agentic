pub struct DeadMemSwapMapItem {
    value: String,
}

impl DeadMemSwapMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-mem-swap-map:{}", self.value)
    }
}

pub fn dead_mem_swap_map(raw: &str) -> String {
    DeadMemSwapMapItem::new(raw).render()
}
