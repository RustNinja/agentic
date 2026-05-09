#[derive(Clone, Debug)]
pub struct VecDrainItem {
    value: String,
}

impl VecDrainItem {
    pub fn live() -> Self {
        Self::new("live")
    }

    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn render_label(&self) -> String {
        format!("vec-drain:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-drain:{}", self.value)
    }
}

fn vec_drain_entries(raw: &str) -> Vec<VecDrainItem> {
    vec![VecDrainItem::live(), VecDrainItem::new(raw)]
}

pub fn selected_vec_drain(raw: &str) -> String {
    let mut entries = vec_drain_entries(raw);
    entries
        .drain(..)
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_vec_drain(raw: &str) -> String {
    VecDrainItem::new(raw).dead_method()
}
