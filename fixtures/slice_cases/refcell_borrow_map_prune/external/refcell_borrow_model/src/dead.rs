pub struct DeadRefCellBorrowMapItem {
    value: String,
}

impl DeadRefCellBorrowMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-refcell-borrow-map:{}", self.value)
    }
}

pub fn dead_refcell_borrow_map(raw: &str) -> String {
    DeadRefCellBorrowMapItem::new(raw).render()
}
