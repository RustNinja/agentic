pub struct DeadRcRefCellBorrowMapItem {
    value: String,
}

impl DeadRcRefCellBorrowMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-rc-refcell-borrow-map:{}", self.value)
    }
}

pub fn dead_rc_refcell_borrow_map(raw: &str) -> String {
    DeadRcRefCellBorrowMapItem::new(raw).render()
}
