use std::cell::RefCell;

pub struct RefCellBorrowMapPayload {
    value: String,
}

impl RefCellBorrowMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("refcell-borrow-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("refcell-borrow-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-refcell-borrow-map:{}", self.value)
    }
}

pub fn selected_refcell_borrow_map(raw: &str) -> String {
    let payload = RefCell::new(RefCellBorrowMapPayload::new(raw));
    let rendered = payload.borrow().render_label();
    rendered
}

pub fn dead_live_refcell_borrow_map(raw: &str) -> String {
    RefCellBorrowMapPayload::new(raw).dead_method()
}
