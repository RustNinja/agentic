use std::cell::RefCell;

pub struct OptionRefCellBorrowMapPayload {
    value: String,
}

impl OptionRefCellBorrowMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-refcell-borrow-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-refcell-borrow-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-refcell-borrow-map:{}", self.value)
    }
}

fn option_refcell_borrow_map_payload(raw: &str) -> Option<RefCell<OptionRefCellBorrowMapPayload>> {
    if raw.trim().is_empty() {
        None
    } else {
        Some(RefCell::new(OptionRefCellBorrowMapPayload::new(raw)))
    }
}

pub fn selected_option_refcell_borrow_map(raw: &str) -> String {
    option_refcell_borrow_map_payload(raw)
        .as_ref()
        .map(|payload| payload.borrow().render_label())
        .unwrap_or_else(|| "option-refcell-borrow-map:missing".to_string())
}

pub fn dead_live_option_refcell_borrow_map(raw: &str) -> String {
    OptionRefCellBorrowMapPayload::new(raw).dead_method()
}
