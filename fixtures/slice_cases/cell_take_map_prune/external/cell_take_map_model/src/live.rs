use std::cell::Cell;
#[derive(Debug, Default)]
pub struct CellTakeMapPayload {
    value: String,
}

impl CellTakeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("cell-take-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("cell-take-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-cell-take-map:{}", self.value)
    }
}

pub fn selected_cell_take_map(raw: &str) -> String {
    let payload = Cell::new(CellTakeMapPayload::new(raw));
    payload.take().render_label()
}

pub fn dead_live_cell_take_map(raw: &str) -> String {
    CellTakeMapPayload::new(raw).unused_label()
}
