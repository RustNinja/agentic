pub struct RetainItem {
    value: String,
}

impl RetainItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn keep(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn sort_key(&self) -> usize {
        self.value.len()
    }

    pub fn render(&self) -> String {
        format!("retain:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-retain:{}", self.value)
    }
}

fn build_items(raw: &str) -> Vec<RetainItem> {
    raw.split(',').map(RetainItem::new).collect()
}

pub fn selected_retain(raw: &str) -> String {
    let mut items = build_items(raw);
    items.retain(|item| item.keep());
    items.sort_by_key(|item| item.sort_key());
    items
        .iter()
        .map(RetainItem::render)
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_retain(raw: &str) -> String {
    RetainItem::new(raw).dead_method()
}
