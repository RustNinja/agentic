pub struct IteratorItem {
    label: String,
}

impl IteratorItem {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.trim().to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("iterator:{}", self.label)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator:{}", self.label)
    }
}

pub fn selected_iterator(raw: &str) -> String {
    [IteratorItem::new(raw), IteratorItem::new("tail")]
        .iter()
        .map(IteratorItem::render)
        .collect::<Vec<_>>()
        .join(",")
}

pub fn dead_live_iterator(raw: &str) -> String {
    IteratorItem::new(raw).dead_method()
}
