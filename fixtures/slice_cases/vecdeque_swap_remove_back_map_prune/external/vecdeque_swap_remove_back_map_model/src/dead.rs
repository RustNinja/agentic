pub struct DeadVecdequeSwapRemoveBackMapItem {
    value: String,
}

impl DeadVecdequeSwapRemoveBackMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-swap-remove-back-map:{}", self.value)
    }
}

pub fn dead_vecdeque_swap_remove_back_map(raw: &str) -> String {
    DeadVecdequeSwapRemoveBackMapItem::new(raw).dead_method()
}
