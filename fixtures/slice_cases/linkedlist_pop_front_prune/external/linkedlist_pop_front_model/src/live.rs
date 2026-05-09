use std::collections::LinkedList;
#[derive(Clone, Debug)]
pub struct LinkedlistPopFrontItem {
    value: String,
}

impl LinkedlistPopFrontItem {
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
        format!("linkedlist-pop-front:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-pop-front:{}", self.value)
    }
}

fn linkedlist_pop_front_entries(raw: &str) -> LinkedList<LinkedlistPopFrontItem> {
    let mut entries = LinkedList::new();
    entries.push_back(LinkedlistPopFrontItem::live());
    entries.push_back(LinkedlistPopFrontItem::new(raw));
    entries
}

pub fn selected_linkedlist_pop_front(raw: &str) -> String {
    let mut entries = linkedlist_pop_front_entries(raw);
    entries
        .pop_front()
        .map(|item| item.render_label())
        .unwrap_or_else(|| "linkedlist-pop-front:missing".to_string())
}

pub fn dead_live_linkedlist_pop_front(raw: &str) -> String {
    LinkedlistPopFrontItem::new(raw).dead_method()
}
