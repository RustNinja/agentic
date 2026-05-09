use std::collections::VecDeque;
#[derive(Clone, Debug)]
pub struct VecdequeIterFindItem {
    value: String,
}

impl VecdequeIterFindItem {
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
        format!("vecdeque-iter-find:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-iter-find:{}", self.value)
    }
}

fn vecdeque_iter_find_entries(raw: &str) -> VecDeque<VecdequeIterFindItem> {
    let mut entries = VecDeque::new();
    entries.push_back(VecdequeIterFindItem::live());
    entries.push_back(VecdequeIterFindItem::new(raw));
    entries
}

pub fn selected_vecdeque_iter_find(raw: &str) -> String {
    let entries = vecdeque_iter_find_entries(raw);
    entries
        .iter()
        .find(|item| item.is_live())
        .map(|item| item.render_label())
        .unwrap_or_else(|| "vecdeque-iter-find:missing".to_string())
}

pub fn dead_live_vecdeque_iter_find(raw: &str) -> String {
    VecdequeIterFindItem::new(raw).dead_method()
}
