pub struct DeadRefcellTryBorrowMapItem;

pub struct DeadRefcellTryBorrowMapPayload {
    value: String,
}

impl DeadRefcellTryBorrowMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-refcell-try-borrow-map:{}", self.value)
    }
}

pub fn dead_refcell_try_borrow_map(raw: &str) -> String {
    DeadRefcellTryBorrowMapPayload::new(raw).dead_method()
}
