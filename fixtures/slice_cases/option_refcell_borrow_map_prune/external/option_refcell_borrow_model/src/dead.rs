pub struct DeadOptionRefCellBorrowMapItem {
    value: String,
}

impl DeadOptionRefCellBorrowMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-refcell-borrow-map:{}", self.value)
    }
}

pub fn dead_option_refcell_borrow_map(raw: &str) -> String {
    DeadOptionRefCellBorrowMapItem::new(raw).render()
}
