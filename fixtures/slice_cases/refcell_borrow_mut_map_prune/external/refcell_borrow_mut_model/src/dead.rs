pub struct DeadRefCellBorrowMutMapItem {
    value: String,
}

impl DeadRefCellBorrowMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-refcell-borrow-mut-map:{}", self.value)
    }
}

pub fn dead_refcell_borrow_mut_map(raw: &str) -> String {
    DeadRefCellBorrowMutMapItem::new(raw).render()
}
