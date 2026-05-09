pub struct DeadSliceSwapIterItem {
    value: String,
}

impl DeadSliceSwapIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-slice-swap-iter:{}", self.value)
    }
}

pub fn dead_slice_swap_iter(raw: &str) -> String {
    DeadSliceSwapIterItem::new(raw).render()
}
