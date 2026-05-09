use std::collections::VecDeque;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct VecDequeMakeContiguousSortItem {
    value: String,
}

impl VecDequeMakeContiguousSortItem {
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
        format!("vecdeque-make-contiguous-sort:{}", self.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-vecdeque-make-contiguous-sort:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-make-contiguous-sort:{}", self.value)
    }
}

fn vecdeque_make_contiguous_sort_entries(raw: &str) -> VecDeque<VecDequeMakeContiguousSortItem> {
    let mut deque = VecDeque::new();
    deque.push_back(VecDequeMakeContiguousSortItem::live());
    deque.push_back(VecDequeMakeContiguousSortItem::other());
    deque.push_back(VecDequeMakeContiguousSortItem::new(raw));
    deque
}

pub fn selected_vecdeque_make_contiguous_sort(raw: &str) -> String {
    let mut entries = vecdeque_make_contiguous_sort_entries(raw);
    entries.make_contiguous().sort_by_key(|item| item.sort_key());
    entries
        .iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_vecdeque_make_contiguous_sort(raw: &str) -> String {
    VecDequeMakeContiguousSortItem::new(raw).dead_method()
}
