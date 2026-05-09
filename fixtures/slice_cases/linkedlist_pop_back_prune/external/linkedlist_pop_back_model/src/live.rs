use std::collections::LinkedList;
#[derive(Clone, Debug)]
pub struct LinkedlistPopBackItem {
    value: String,
}

impl LinkedlistPopBackItem {
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
        format!("linkedlist-pop-back:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-pop-back:{}", self.value)
    }
}

fn linkedlist_pop_back_entries(raw: &str) -> LinkedList<LinkedlistPopBackItem> {
    let mut entries = LinkedList::new();
    entries.push_back(LinkedlistPopBackItem::new(raw));
    entries.push_back(LinkedlistPopBackItem::live());
    entries
}

pub fn selected_linkedlist_pop_back(raw: &str) -> String {
    let mut entries = linkedlist_pop_back_entries(raw);
    entries
        .pop_back()
        .map(|item| item.render_label())
        .unwrap_or_else(|| "linkedlist-pop-back:missing".to_string())
}

pub fn dead_live_linkedlist_pop_back(raw: &str) -> String {
    LinkedlistPopBackItem::new(raw).dead_method()
}
