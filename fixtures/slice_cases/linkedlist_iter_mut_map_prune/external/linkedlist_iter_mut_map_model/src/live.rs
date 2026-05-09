use std::collections::LinkedList;
pub struct LinkedlistIterMutMapPayload {
    value: String,
}

impl LinkedlistIterMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("linkedlist-iter-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("linkedlist-iter-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-linkedlist-iter-mut-map:{}", self.value)
    }
}

pub fn selected_linkedlist_iter_mut_map(raw: &str) -> String {
    let mut values = LinkedList::new();
    values.push_back(LinkedlistIterMutMapPayload::new(raw));
    values
        .iter_mut()
        .map(|payload| payload.bump_and_render())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_linkedlist_iter_mut_map(raw: &str) -> String {
    LinkedlistIterMutMapPayload::new(raw).unused_label()
}
