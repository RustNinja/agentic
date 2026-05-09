use std::collections::LinkedList;
#[derive(Clone, Debug)]
pub struct LinkedlistFrontItem {
    value: String,
}

impl LinkedlistFrontItem {
    pub fn live() -> Self {
        Self::new("live")
    }

    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn render_label(&self) -> String {
        format!("linkedlist-front:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-front:{}", self.value)
    }
}

fn linkedlist_front_entries(raw: &str) -> LinkedList<LinkedlistFrontItem> {
    let mut entries = LinkedList::new();
    entries.push_back(LinkedlistFrontItem::live());
    entries.push_back(LinkedlistFrontItem::new(raw));
    entries
}

pub fn selected_linkedlist_front(raw: &str) -> String {
    let entries = linkedlist_front_entries(raw);
    entries
        .front()
        .map(|item| item.render_label())
        .unwrap_or_else(|| "linkedlist-front:missing".to_string())
}

pub fn dead_live_linkedlist_front(raw: &str) -> String {
    LinkedlistFrontItem::new(raw).dead_method()
}
