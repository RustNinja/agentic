use std::collections::VecDeque;
pub struct VecdequeRemoveMapPayload {
    value: String,
}

impl VecdequeRemoveMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-remove-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-remove-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-remove-map:{}", self.value)
    }
}

pub fn selected_vecdeque_remove_map(raw: &str) -> String {
    let mut values = VecDeque::from([VecdequeRemoveMapPayload::new(raw)]);
    values
        .remove(0)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vecdeque_remove_map(raw: &str) -> String {
    VecdequeRemoveMapPayload::new(raw).unused_label()
}
