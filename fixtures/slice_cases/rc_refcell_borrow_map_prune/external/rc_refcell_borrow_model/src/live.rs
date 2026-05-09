use std::cell::RefCell;
use std::rc::Rc;

pub struct RcRefCellBorrowMapPayload {
    value: String,
}

impl RcRefCellBorrowMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rc-refcell-borrow-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("rc-refcell-borrow-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rc-refcell-borrow-map:{}", self.value)
    }
}

pub fn selected_rc_refcell_borrow_map(raw: &str) -> String {
    let payload = Rc::new(RefCell::new(RcRefCellBorrowMapPayload::new(raw)));
    let rendered = payload.borrow().render_label();
    rendered
}

pub fn dead_live_rc_refcell_borrow_map(raw: &str) -> String {
    RcRefCellBorrowMapPayload::new(raw).dead_method()
}
