#[derive(Clone)]
pub struct ReduceItem {
    value: String,
}

impl ReduceItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.value.push('+');
        self.value.push_str(&other.value);
        self
    }

    pub fn render(&self) -> String {
        format!("reduce:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-reduce:{}", self.value)
    }
}

fn build_items(raw: &str) -> Vec<ReduceItem> {
    raw.split(',').map(ReduceItem::new).collect()
}

pub fn selected_reduce(raw: &str) -> String {
    build_items(raw)
        .iter()
        .cloned()
        .reduce(|left, right| left.merge(right))
        .map(|item| item.render())
        .unwrap_or_else(|| "reduce:none".to_string())
}

pub fn dead_live_reduce(raw: &str) -> String {
    ReduceItem::new(raw).dead_method()
}
