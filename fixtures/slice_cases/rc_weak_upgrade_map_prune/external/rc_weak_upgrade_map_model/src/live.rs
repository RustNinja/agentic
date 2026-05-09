use std::rc::{Rc, Weak};

#[derive(Clone)]
pub struct RcWeakUpgradeMapPayload {
    value: String,
}

impl RcWeakUpgradeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rc-weak-upgrade-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("rc-weak-upgrade-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rc-weak-upgrade-map:{}", self.value)
    }
}

fn rc_weak_upgrade_map_payload(
    raw: &str,
) -> (Rc<RcWeakUpgradeMapPayload>, Weak<RcWeakUpgradeMapPayload>) {
    let payload = Rc::new(RcWeakUpgradeMapPayload::new(raw));
    let weak = Rc::downgrade(&payload);
    (payload, weak)
}

pub fn selected_rc_weak_upgrade_map(raw: &str) -> String {
    let (_strong, weak) = rc_weak_upgrade_map_payload(raw);
    weak.upgrade()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "rc-weak-upgrade-map:missing".to_string())
}

pub fn dead_live_rc_weak_upgrade_map(raw: &str) -> String {
    RcWeakUpgradeMapPayload::new(raw).dead_method()
}
