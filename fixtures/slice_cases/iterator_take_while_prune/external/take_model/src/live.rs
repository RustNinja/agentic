pub struct TakeItem {
    value: String,
}

impl TakeItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn keep_prefix(&self) -> bool {
        !self.value.is_empty() && self.value != "stop"
    }

    pub fn render(&self) -> String {
        format!("take:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-take:{}", self.value)
    }
}

fn build_items(raw: &str) -> Vec<TakeItem> {
    raw.split(',').map(TakeItem::new).collect()
}

pub fn selected_take_while(raw: &str) -> String {
    build_items(raw)
        .iter()
        .take_while(|item| item.keep_prefix())
        .map(|item| item.render())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_take_while(raw: &str) -> String {
    TakeItem::new(raw).dead_method()
}
