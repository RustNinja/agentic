#[derive(Clone)]
pub struct ClonedItem {
    value: String,
}

impl ClonedItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("cloned:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-cloned:{}", self.value)
    }
}

fn build_items(raw: &str) -> Vec<ClonedItem> {
    raw.split(',').map(ClonedItem::new).collect()
}

pub fn selected_cloned(raw: &str) -> String {
    build_items(raw)
        .iter()
        .cloned()
        .map(|item| item.render())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_cloned(raw: &str) -> String {
    ClonedItem::new(raw).dead_method()
}
