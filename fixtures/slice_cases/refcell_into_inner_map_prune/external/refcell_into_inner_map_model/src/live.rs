use std::cell::RefCell;
pub struct RefcellIntoInnerMapPayload {
    value: String,
}

impl RefcellIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("refcell-into-inner-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("refcell-into-inner-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-refcell-into-inner-map:{}", self.value)
    }
}

pub fn selected_refcell_into_inner_map(raw: &str) -> String {
    let payload = RefCell::new(RefcellIntoInnerMapPayload::new(raw));
    payload.into_inner().render_label()
}

pub fn dead_live_refcell_into_inner_map(raw: &str) -> String {
    RefcellIntoInnerMapPayload::new(raw).unused_label()
}
