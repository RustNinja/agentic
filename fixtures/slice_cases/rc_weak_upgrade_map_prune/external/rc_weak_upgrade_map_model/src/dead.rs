pub struct DeadRcWeakUpgradeMapItem {
    value: String,
}

impl DeadRcWeakUpgradeMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-rc-weak-upgrade-map:{}", self.value)
    }
}

pub fn dead_rc_weak_upgrade_map(raw: &str) -> String {
    DeadRcWeakUpgradeMapItem::new(raw).render()
}
