use std::cell::RefCell;
pub struct RefcellTryBorrowMutMapPayload {
    value: String,
}

impl RefcellTryBorrowMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("refcell-try-borrow-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("refcell-try-borrow-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-refcell-try-borrow-mut-map:{}", self.value)
    }
}

pub fn selected_refcell_try_borrow_mut_map(raw: &str) -> String {
    let payload = RefCell::new(RefcellTryBorrowMutMapPayload::new(raw));
    payload
        .try_borrow_mut()
        .map(|mut payload| payload.bump_and_render())
        .unwrap_or_default()
}

pub fn dead_live_refcell_try_borrow_mut_map(raw: &str) -> String {
    RefcellTryBorrowMutMapPayload::new(raw).unused_label()
}
