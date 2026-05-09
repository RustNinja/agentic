use std::collections::VecDeque;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct VecDequeRetainItem {
    value: String,
}

impl VecDequeRetainItem {
    pub fn live() -> Self {
        Self::new("live")
    }

    pub fn other() -> Self {
        Self::new("other")
    }

    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.value.contains("live")
    }

    pub fn bump(&mut self) -> &mut Self {
        self.value.push_str("-live");
        self
    }

    pub fn sort_key(&self) -> usize {
        self.value.len()
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-retain:{}", self.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-vecdeque-retain:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-retain:{}", self.value)
    }
}

fn vecdeque_retain_entries(raw: &str) -> VecDeque<VecDequeRetainItem> {
    let mut deque = VecDeque::new();
    deque.push_back(VecDequeRetainItem::live());
    deque.push_back(VecDequeRetainItem::other());
    deque.push_back(VecDequeRetainItem::new(raw));
    deque
}

pub fn selected_vecdeque_retain(raw: &str) -> String {
    let mut entries = vecdeque_retain_entries(raw);
    entries.retain(|item| item.is_live());
    entries
        .iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_vecdeque_retain(raw: &str) -> String {
    VecDequeRetainItem::new(raw).dead_method()
}
