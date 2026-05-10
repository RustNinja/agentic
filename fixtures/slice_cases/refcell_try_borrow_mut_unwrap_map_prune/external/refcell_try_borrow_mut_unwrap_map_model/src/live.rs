use std::cell::RefCell;
pub struct RefcellTryBorrowMutUnwrapMapPayload {
    value: String,
}

impl RefcellTryBorrowMutUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("refcell-try-borrow-mut-unwrap-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("refcell-try-borrow-mut-unwrap-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-refcell-try-borrow-mut-unwrap-map:{}", self.value)
    }
}

pub fn selected_refcell_try_borrow_mut_unwrap_map(raw: &str) -> String {
    let payload = RefCell::new(RefcellTryBorrowMutUnwrapMapPayload::new(raw));
    let label = payload.try_borrow_mut().unwrap().bump_and_render();
    label
}

pub fn dead_live_refcell_try_borrow_mut_unwrap_map(raw: &str) -> String {
    RefcellTryBorrowMutUnwrapMapPayload::new(raw).unused_label()
}
