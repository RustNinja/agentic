use std::collections::VecDeque;
#[derive(Clone, Debug)]
pub struct VecdequeIntoIterItem {
    value: String,
}

impl VecdequeIntoIterItem {
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
        format!("vecdeque-into-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-into-iter:{}", self.value)
    }
}

fn vecdeque_into_iter_entries(raw: &str) -> VecDeque<VecdequeIntoIterItem> {
    let mut entries = VecDeque::new();
    entries.push_back(VecdequeIntoIterItem::live());
    entries.push_back(VecdequeIntoIterItem::new(raw));
    entries
}

pub fn selected_vecdeque_into_iter(raw: &str) -> String {
    let entries = vecdeque_into_iter_entries(raw);
    entries
        .into_iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_vecdeque_into_iter(raw: &str) -> String {
    VecdequeIntoIterItem::new(raw).dead_method()
}
