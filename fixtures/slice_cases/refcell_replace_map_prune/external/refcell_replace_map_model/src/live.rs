use std::cell::RefCell;

#[derive(Clone)]
pub struct RefCellReplaceMapPayload {
    value: String,
}

impl RefCellReplaceMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("refcell-replace-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("refcell-replace-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-refcell-replace-map:{}", self.value)
    }
}

pub fn selected_refcell_replace_map(raw: &str) -> String {
    let cell = RefCell::new(RefCellReplaceMapPayload::new(raw));
    cell.replace(RefCellReplaceMapPayload::new("replacement"))
        .render_label()
}

pub fn dead_live_refcell_replace_map(raw: &str) -> String {
    RefCellReplaceMapPayload::new(raw).dead_method()
}
