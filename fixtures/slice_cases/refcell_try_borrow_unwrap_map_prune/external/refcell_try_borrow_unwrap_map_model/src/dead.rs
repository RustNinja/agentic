pub struct DeadRefcellTryBorrowUnwrapMapItem;

pub struct DeadRefcellTryBorrowUnwrapMapPayload {
    value: String,
}

impl DeadRefcellTryBorrowUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-refcell-try-borrow-unwrap-map:{}", self.value)
    }
}

pub fn dead_refcell_try_borrow_unwrap_map(raw: &str) -> String {
    DeadRefcellTryBorrowUnwrapMapPayload::new(raw).dead_method()
}
