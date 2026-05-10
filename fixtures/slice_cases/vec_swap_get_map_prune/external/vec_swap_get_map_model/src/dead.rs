pub struct DeadVecSwapGetMapItem {
    value: String,
}

impl DeadVecSwapGetMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-swap-get-map:{}", self.value)
    }
}

pub fn dead_vec_swap_get_map(raw: &str) -> String {
    DeadVecSwapGetMapItem::new(raw).dead_method()
}
