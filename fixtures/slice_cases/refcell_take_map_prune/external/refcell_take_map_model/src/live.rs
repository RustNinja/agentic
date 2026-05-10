use std::cell::RefCell;
#[derive(Debug, Default)]
pub struct RefcellTakeMapPayload {
    value: String,
}

impl RefcellTakeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("refcell-take-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("refcell-take-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-refcell-take-map:{}", self.value)
    }
}

pub fn selected_refcell_take_map(raw: &str) -> String {
    let payload = RefCell::new(RefcellTakeMapPayload::new(raw));
    payload.take().render_label()
}

pub fn dead_live_refcell_take_map(raw: &str) -> String {
    RefcellTakeMapPayload::new(raw).unused_label()
}
