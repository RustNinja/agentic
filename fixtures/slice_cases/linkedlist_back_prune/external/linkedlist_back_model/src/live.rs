use std::collections::LinkedList;
#[derive(Clone, Debug)]
pub struct LinkedlistBackItem {
    value: String,
}

impl LinkedlistBackItem {
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
        format!("linkedlist-back:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-back:{}", self.value)
    }
}

fn linkedlist_back_entries(raw: &str) -> LinkedList<LinkedlistBackItem> {
    let mut entries = LinkedList::new();
    entries.push_back(LinkedlistBackItem::new(raw));
    entries.push_back(LinkedlistBackItem::live());
    entries
}

pub fn selected_linkedlist_back(raw: &str) -> String {
    let entries = linkedlist_back_entries(raw);
    entries
        .back()
        .map(|item| item.render_label())
        .unwrap_or_else(|| "linkedlist-back:missing".to_string())
}

pub fn dead_live_linkedlist_back(raw: &str) -> String {
    LinkedlistBackItem::new(raw).dead_method()
}
