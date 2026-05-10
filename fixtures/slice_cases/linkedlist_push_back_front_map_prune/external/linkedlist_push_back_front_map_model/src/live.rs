use std::collections::LinkedList;
pub struct LinkedlistPushBackFrontMapPayload {
    value: String,
}

impl LinkedlistPushBackFrontMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("linkedlist-push-back-front-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("linkedlist-push-back-front-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-linkedlist-push-back-front-map:{}", self.value)
    }
}

pub fn selected_linkedlist_push_back_front_map(raw: &str) -> String {
    let mut values = LinkedList::new();
    values.push_back(LinkedlistPushBackFrontMapPayload::new(raw));
    values
        .front()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_linkedlist_push_back_front_map(raw: &str) -> String {
    LinkedlistPushBackFrontMapPayload::new(raw).unused_label()
}
