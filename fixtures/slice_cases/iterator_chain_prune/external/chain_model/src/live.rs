pub struct ChainItem {
    value: String,
}

impl ChainItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("chain:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-chain:{}", self.value)
    }
}

fn primary_items(raw: &str) -> Vec<ChainItem> {
    raw.split(',').map(ChainItem::new).collect()
}

fn fallback_items() -> Vec<ChainItem> {
    vec![ChainItem::new("fallback")]
}

pub fn selected_chain(raw: &str) -> String {
    let primary = primary_items(raw);
    let fallback = fallback_items();
    primary
        .iter()
        .chain(fallback.iter())
        .map(|item| item.render())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_chain(raw: &str) -> String {
    ChainItem::new(raw).dead_method()
}
