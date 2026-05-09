use std::collections::LinkedList;
#[derive(Clone, Debug)]
pub struct LinkedlistIterFindItem {
    value: String,
}

impl LinkedlistIterFindItem {
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
        format!("linkedlist-iter-find:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-iter-find:{}", self.value)
    }
}

fn linkedlist_iter_find_entries(raw: &str) -> LinkedList<LinkedlistIterFindItem> {
    let mut entries = LinkedList::new();
    entries.push_back(LinkedlistIterFindItem::live());
    entries.push_back(LinkedlistIterFindItem::new(raw));
    entries
}

pub fn selected_linkedlist_iter_find(raw: &str) -> String {
    let entries = linkedlist_iter_find_entries(raw);
    entries
        .iter()
        .find(|item| item.is_live())
        .map(|item| item.render_label())
        .unwrap_or_else(|| "linkedlist-iter-find:missing".to_string())
}

pub fn dead_live_linkedlist_iter_find(raw: &str) -> String {
    LinkedlistIterFindItem::new(raw).dead_method()
}
