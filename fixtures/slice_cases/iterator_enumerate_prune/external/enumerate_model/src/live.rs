pub struct EnumerateItem {
    value: String,
}

impl EnumerateItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_at(&self, index: usize) -> String {
        format!("enumerate:{index}:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-enumerate:{}", self.value)
    }
}

fn build_items(raw: &str) -> Vec<EnumerateItem> {
    raw.split(',').map(EnumerateItem::new).collect()
}

pub fn selected_enumerate(raw: &str) -> String {
    build_items(raw)
        .iter()
        .enumerate()
        .map(|(index, item)| item.render_at(index))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_enumerate(raw: &str) -> String {
    EnumerateItem::new(raw).dead_method()
}
