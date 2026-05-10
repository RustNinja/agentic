use std::cell::Cell;
pub struct CellSetGetMapPayload {
    value: String,
}

impl CellSetGetMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("cell-set-get-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("cell-set-get-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-cell-set-get-map:{}", self.value)
    }
}

pub fn selected_cell_set_get_map(raw: &str) -> String {
    let value = Cell::new(raw.len());
    value.set(value.get() + 1);
    CellSetGetMapPayload::new(&value.get().to_string()).render_label()
}

pub fn dead_live_cell_set_get_map(raw: &str) -> String {
    CellSetGetMapPayload::new(raw).unused_label()
}
