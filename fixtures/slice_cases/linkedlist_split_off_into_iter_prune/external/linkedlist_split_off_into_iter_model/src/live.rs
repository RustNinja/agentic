use std::collections::LinkedList;

#[derive(Clone)]
pub struct LinkedlistSplitOffIntoIterPayload {
    value: String,
}

impl LinkedlistSplitOffIntoIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("linkedlist-split-off-into-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("linkedlist-split-off-into-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-split-off-into-iter:{}", self.value)
    }
}

fn linkedlist_split_off_into_iter_payloads(
    raw: &str,
) -> LinkedList<LinkedlistSplitOffIntoIterPayload> {
    let mut payloads = LinkedList::new();
    payloads.push_back(LinkedlistSplitOffIntoIterPayload::new(raw));
    payloads.push_back(LinkedlistSplitOffIntoIterPayload::new("tail"));
    payloads
}

pub fn selected_linkedlist_split_off_into_iter(raw: &str) -> String {
    let mut payloads = linkedlist_split_off_into_iter_payloads(raw);
    payloads
        .split_off(1)
        .into_iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "linkedlist-split-off-into-iter:missing".to_string())
}

pub fn dead_live_linkedlist_split_off_into_iter(raw: &str) -> String {
    LinkedlistSplitOffIntoIterPayload::new(raw).dead_method()
}
