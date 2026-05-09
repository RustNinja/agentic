use std::cell::Cell;

#[derive(Clone)]
pub struct CellReplaceMapPayload {
    value: String,
}

impl CellReplaceMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("cell-replace-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("cell-replace-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-cell-replace-map:{}", self.value)
    }
}

pub fn selected_cell_replace_map(raw: &str) -> String {
    let cell = Cell::new(CellReplaceMapPayload::new(raw));
    cell.replace(CellReplaceMapPayload::new("replacement"))
        .render_label()
}

pub fn dead_live_cell_replace_map(raw: &str) -> String {
    CellReplaceMapPayload::new(raw).dead_method()
}
