#[derive(Clone, Debug)]
pub struct VecIntoIterItem {
    value: String,
}

impl VecIntoIterItem {
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
        format!("vec-into-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-into-iter:{}", self.value)
    }
}

fn vec_into_iter_entries(raw: &str) -> Vec<VecIntoIterItem> {
    vec![VecIntoIterItem::live(), VecIntoIterItem::new(raw)]
}

pub fn selected_vec_into_iter(raw: &str) -> String {
    let entries = vec_into_iter_entries(raw);
    entries
        .into_iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_vec_into_iter(raw: &str) -> String {
    VecIntoIterItem::new(raw).dead_method()
}
