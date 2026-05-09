pub struct DeadVecSwapIterItem {
    value: String,
}

impl DeadVecSwapIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-swap-iter:{}", self.value)
    }
}

pub fn dead_vec_swap_iter(raw: &str) -> String {
    DeadVecSwapIterItem::new(raw).render()
}
