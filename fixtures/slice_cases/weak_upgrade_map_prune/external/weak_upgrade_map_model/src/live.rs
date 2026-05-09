use std::sync::{Arc, Weak};

#[derive(Clone)]
pub struct WeakUpgradeMapPayload {
    value: String,
}

impl WeakUpgradeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("weak-upgrade-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("weak-upgrade-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-weak-upgrade-map:{}", self.value)
    }
}

fn weak_upgrade_map_payload(
    raw: &str,
) -> (Arc<WeakUpgradeMapPayload>, Weak<WeakUpgradeMapPayload>) {
    let payload = Arc::new(WeakUpgradeMapPayload::new(raw));
    let weak = Arc::downgrade(&payload);
    (payload, weak)
}

pub fn selected_weak_upgrade_map(raw: &str) -> String {
    let (_strong, weak) = weak_upgrade_map_payload(raw);
    weak.upgrade()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "weak-upgrade-map:missing".to_string())
}

pub fn dead_live_weak_upgrade_map(raw: &str) -> String {
    WeakUpgradeMapPayload::new(raw).dead_method()
}
