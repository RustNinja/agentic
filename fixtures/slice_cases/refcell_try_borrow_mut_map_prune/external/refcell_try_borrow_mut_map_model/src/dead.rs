pub struct DeadRefcellTryBorrowMutMapItem;

pub struct DeadRefcellTryBorrowMutMapPayload {
    value: String,
}

impl DeadRefcellTryBorrowMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-refcell-try-borrow-mut-map:{}", self.value)
    }
}

pub fn dead_refcell_try_borrow_mut_map(raw: &str) -> String {
    DeadRefcellTryBorrowMutMapPayload::new(raw).dead_method()
}
