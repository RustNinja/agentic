use std::collections::LinkedList;
#[derive(Clone, Debug)]
pub struct LinkedlistIntoIterItem {
    value: String,
}

impl LinkedlistIntoIterItem {
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
        format!("linkedlist-into-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-into-iter:{}", self.value)
    }
}

fn linkedlist_into_iter_entries(raw: &str) -> LinkedList<LinkedlistIntoIterItem> {
    let mut entries = LinkedList::new();
    entries.push_back(LinkedlistIntoIterItem::live());
    entries.push_back(LinkedlistIntoIterItem::new(raw));
    entries
}

pub fn selected_linkedlist_into_iter(raw: &str) -> String {
    let entries = linkedlist_into_iter_entries(raw);
    entries
        .into_iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_linkedlist_into_iter(raw: &str) -> String {
    LinkedlistIntoIterItem::new(raw).dead_method()
}
