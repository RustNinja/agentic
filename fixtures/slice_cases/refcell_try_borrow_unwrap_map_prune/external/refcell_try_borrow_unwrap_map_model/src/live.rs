use std::cell::RefCell;
pub struct RefcellTryBorrowUnwrapMapPayload {
    value: String,
}

impl RefcellTryBorrowUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("refcell-try-borrow-unwrap-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("refcell-try-borrow-unwrap-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-refcell-try-borrow-unwrap-map:{}", self.value)
    }
}

pub fn selected_refcell_try_borrow_unwrap_map(raw: &str) -> String {
    let payload = RefCell::new(RefcellTryBorrowUnwrapMapPayload::new(raw));
    let label = payload.try_borrow().unwrap().render_label();
    label
}

pub fn dead_live_refcell_try_borrow_unwrap_map(raw: &str) -> String {
    RefcellTryBorrowUnwrapMapPayload::new(raw).unused_label()
}
