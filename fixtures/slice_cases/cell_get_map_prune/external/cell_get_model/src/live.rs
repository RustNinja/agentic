use std::cell::Cell;

#[derive(Clone, Copy)]
pub struct CellGetMapPayload {
    value: i32,
}

impl CellGetMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().len() as i32,
        }
    }

    pub fn render_label(&self) -> String {
        format!("cell-get-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value += 1;
        format!("cell-get-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-cell-get-map:{}", self.value)
    }
}

pub fn selected_cell_get_map(raw: &str) -> String {
    let payload = Cell::new(CellGetMapPayload::new(raw));
    payload.get().render_label()
}

pub fn dead_live_cell_get_map(raw: &str) -> String {
    CellGetMapPayload::new(raw).dead_method()
}
