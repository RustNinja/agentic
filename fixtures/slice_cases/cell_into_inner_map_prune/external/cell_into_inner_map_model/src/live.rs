use std::cell::Cell;
pub struct CellIntoInnerMapPayload {
    value: String,
}

impl CellIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("cell-into-inner-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("cell-into-inner-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-cell-into-inner-map:{}", self.value)
    }
}

pub fn selected_cell_into_inner_map(raw: &str) -> String {
    let payload = Cell::new(CellIntoInnerMapPayload::new(raw));
    payload.into_inner().render_label()
}

pub fn dead_live_cell_into_inner_map(raw: &str) -> String {
    CellIntoInnerMapPayload::new(raw).unused_label()
}
