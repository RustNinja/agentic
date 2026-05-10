use std::collections::LinkedList;
pub struct LinkedlistPushFrontBackMapPayload {
    value: String,
}

impl LinkedlistPushFrontBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("linkedlist-push-front-back-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("linkedlist-push-front-back-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-linkedlist-push-front-back-map:{}", self.value)
    }
}

pub fn selected_linkedlist_push_front_back_map(raw: &str) -> String {
    let mut values = LinkedList::new();
    values.push_front(LinkedlistPushFrontBackMapPayload::new(raw));
    values
        .back()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_linkedlist_push_front_back_map(raw: &str) -> String {
    LinkedlistPushFrontBackMapPayload::new(raw).unused_label()
}
