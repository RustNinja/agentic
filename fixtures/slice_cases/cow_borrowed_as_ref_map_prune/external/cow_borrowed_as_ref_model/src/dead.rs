pub struct DeadCowBorrowedAsRefMapItem {
    value: String,
}

impl DeadCowBorrowedAsRefMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-cow-borrowed-as-ref-map:{}", self.value)
    }
}

pub fn dead_cow_borrowed_as_ref_map(raw: &str) -> String {
    DeadCowBorrowedAsRefMapItem::new(raw).render()
}
