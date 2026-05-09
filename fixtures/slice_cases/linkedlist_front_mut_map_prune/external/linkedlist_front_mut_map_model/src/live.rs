use std::collections::LinkedList;
pub struct LinkedlistFrontMutMapPayload {
    value: String,
}

impl LinkedlistFrontMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("linkedlist-front-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("linkedlist-front-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-linkedlist-front-mut-map:{}", self.value)
    }
}

pub fn selected_linkedlist_front_mut_map(raw: &str) -> String {
    let mut values = LinkedList::new();
    values.push_back(LinkedlistFrontMutMapPayload::new(raw));
    values
        .front_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_linkedlist_front_mut_map(raw: &str) -> String {
    LinkedlistFrontMutMapPayload::new(raw).unused_label()
}
