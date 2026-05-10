use std::cell::RefCell;
pub struct RefcellGetMutMapPayload {
    value: String,
}

impl RefcellGetMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("refcell-get-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("refcell-get-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-refcell-get-mut-map:{}", self.value)
    }
}

pub fn selected_refcell_get_mut_map(raw: &str) -> String {
    let mut payload = RefCell::new(RefcellGetMutMapPayload::new(raw));
    payload.get_mut().bump_and_render()
}

pub fn dead_live_refcell_get_mut_map(raw: &str) -> String {
    RefcellGetMutMapPayload::new(raw).unused_label()
}
