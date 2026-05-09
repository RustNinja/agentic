use std::collections::VecDeque;
#[derive(Clone, Debug)]
pub struct VecdequeDrainItem {
    value: String,
}

impl VecdequeDrainItem {
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
        format!("vecdeque-drain:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-drain:{}", self.value)
    }
}

fn vecdeque_drain_entries(raw: &str) -> VecDeque<VecdequeDrainItem> {
    let mut entries = VecDeque::new();
    entries.push_back(VecdequeDrainItem::live());
    entries.push_back(VecdequeDrainItem::new(raw));
    entries
}

pub fn selected_vecdeque_drain(raw: &str) -> String {
    let mut entries = vecdeque_drain_entries(raw);
    entries
        .drain(..)
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_vecdeque_drain(raw: &str) -> String {
    VecdequeDrainItem::new(raw).dead_method()
}
