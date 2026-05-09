pub struct DeadBorrowTraitMapItem {
    value: String,
}

impl DeadBorrowTraitMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-borrow-trait-map:{}", self.value)
    }
}

pub fn dead_borrow_trait_map(raw: &str) -> String {
    DeadBorrowTraitMapItem::new(raw).render()
}
