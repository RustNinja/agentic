use std::collections::LinkedList;
pub struct LinkedlistAppendBackMapPayload {
    value: String,
}

impl LinkedlistAppendBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("linkedlist-append-back-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("linkedlist-append-back-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-linkedlist-append-back-map:{}", self.value)
    }
}

pub fn selected_linkedlist_append_back_map(raw: &str) -> String {
    let mut values = LinkedList::new();
    values.push_back(LinkedlistAppendBackMapPayload::new("old"));
    let mut extras = LinkedList::new();
    extras.push_back(LinkedlistAppendBackMapPayload::new(raw));
    values.append(&mut extras);
    values
        .back()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_linkedlist_append_back_map(raw: &str) -> String {
    LinkedlistAppendBackMapPayload::new(raw).unused_label()
}
