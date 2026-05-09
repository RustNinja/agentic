pub struct DeadVecdequeSwapFrontMapItem {
    value: String,
}

impl DeadVecdequeSwapFrontMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-swap-front-map:{}", self.value)
    }
}

pub fn dead_vecdeque_swap_front_map(raw: &str) -> String {
    DeadVecdequeSwapFrontMapItem::new(raw).dead_method()
}
