use std::collections::LinkedList;
pub struct LinkedlistBackMutMapPayload {
    value: String,
}

impl LinkedlistBackMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("linkedlist-back-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("linkedlist-back-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-linkedlist-back-mut-map:{}", self.value)
    }
}

pub fn selected_linkedlist_back_mut_map(raw: &str) -> String {
    let mut values = LinkedList::new();
    values.push_back(LinkedlistBackMutMapPayload::new(raw));
    values
        .back_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_linkedlist_back_mut_map(raw: &str) -> String {
    LinkedlistBackMutMapPayload::new(raw).unused_label()
}
