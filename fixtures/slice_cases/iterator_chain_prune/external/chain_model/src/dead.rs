pub struct DeadChainItem {
    value: String,
}

impl DeadChainItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-chain:{}", self.value)
    }
}

pub fn dead_chain(raw: &str) -> String {
    DeadChainItem::new(raw).render()
}
