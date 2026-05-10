pub struct DeadRefcellTryBorrowMutUnwrapMapItem;

pub struct DeadRefcellTryBorrowMutUnwrapMapPayload {
    value: String,
}

impl DeadRefcellTryBorrowMutUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-refcell-try-borrow-mut-unwrap-map:{}", self.value)
    }
}

pub fn dead_refcell_try_borrow_mut_unwrap_map(raw: &str) -> String {
    DeadRefcellTryBorrowMutUnwrapMapPayload::new(raw).dead_method()
}
