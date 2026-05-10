use std::cell::RefCell;
pub struct RefcellTryBorrowMapPayload {
    value: String,
}

impl RefcellTryBorrowMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("refcell-try-borrow-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("refcell-try-borrow-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-refcell-try-borrow-map:{}", self.value)
    }
}

pub fn selected_refcell_try_borrow_map(raw: &str) -> String {
    let payload = RefCell::new(RefcellTryBorrowMapPayload::new(raw));
    payload
        .try_borrow()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_refcell_try_borrow_map(raw: &str) -> String {
    RefcellTryBorrowMapPayload::new(raw).unused_label()
}
