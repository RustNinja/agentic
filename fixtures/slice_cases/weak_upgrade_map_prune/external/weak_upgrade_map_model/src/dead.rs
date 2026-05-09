pub struct DeadWeakUpgradeMapItem {
    value: String,
}

impl DeadWeakUpgradeMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-weak-upgrade-map:{}", self.value)
    }
}

pub fn dead_weak_upgrade_map(raw: &str) -> String {
    DeadWeakUpgradeMapItem::new(raw).render()
}
