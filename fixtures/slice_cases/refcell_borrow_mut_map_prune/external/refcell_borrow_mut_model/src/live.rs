use std::cell::RefCell;

pub struct RefCellBorrowMutMapPayload {
    value: String,
}

impl RefCellBorrowMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("refcell-borrow-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("refcell-borrow-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-refcell-borrow-mut-map:{}", self.value)
    }
}

pub fn selected_refcell_borrow_mut_map(raw: &str) -> String {
    let payload = RefCell::new(RefCellBorrowMutMapPayload::new(raw));
    let rendered = payload.borrow_mut().bump_and_render();
    rendered
}

pub fn dead_live_refcell_borrow_mut_map(raw: &str) -> String {
    RefCellBorrowMutMapPayload::new(raw).dead_method()
}
