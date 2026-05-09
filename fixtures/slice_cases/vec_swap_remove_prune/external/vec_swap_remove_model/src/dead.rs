pub struct DeadVecSwapRemoveItem {
    value: String,
}

impl DeadVecSwapRemoveItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-swap-remove:{}", self.value)
    }
}

pub fn dead_vec_swap_remove(raw: &str) -> String {
    DeadVecSwapRemoveItem::new(raw).dead_method()
}
