#[derive(Clone, Debug)]
pub struct VecIterFindItem {
    value: String,
}

impl VecIterFindItem {
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
        format!("vec-iter-find:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-iter-find:{}", self.value)
    }
}

fn vec_iter_find_entries(raw: &str) -> Vec<VecIterFindItem> {
    vec![VecIterFindItem::live(), VecIterFindItem::new(raw)]
}

pub fn selected_vec_iter_find(raw: &str) -> String {
    let entries = vec_iter_find_entries(raw);
    entries
        .iter()
        .find(|item| item.is_live())
        .map(|item| item.render_label())
        .unwrap_or_else(|| "vec-iter-find:missing".to_string())
}

pub fn dead_live_vec_iter_find(raw: &str) -> String {
    VecIterFindItem::new(raw).dead_method()
}
