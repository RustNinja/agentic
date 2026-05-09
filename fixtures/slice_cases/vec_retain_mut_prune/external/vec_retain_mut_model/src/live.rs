use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct VecRetainMutItem {
    value: String,
}

impl VecRetainMutItem {
    pub fn live() -> Self {
        Self::new("live")
    }

    pub fn other() -> Self {
        Self::new("other")
    }

    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.value.contains("live")
    }

    pub fn bump(&mut self) -> &mut Self {
        self.value.push_str("-live");
        self
    }

    pub fn sort_key(&self) -> usize {
        self.value.len()
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn compare_key(&self, raw: &str) -> Ordering {
        self.value.len().cmp(&raw.len())
    }

    pub fn render_label(&self) -> String {
        format!("vec-retain-mut:{}", self.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-vec-retain-mut:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-retain-mut:{}", self.value)
    }
}

fn vec_retain_mut_entries(raw: &str) -> Vec<VecRetainMutItem> {
    vec![VecRetainMutItem::live(), VecRetainMutItem::other(), VecRetainMutItem::new(raw)]
}

pub fn selected_vec_retain_mut(raw: &str) -> String {
    let mut items = vec_retain_mut_entries(raw);
    items.retain_mut(|item| item.bump().is_live());
    items
        .iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_vec_retain_mut(raw: &str) -> String {
    VecRetainMutItem::new(raw).dead_method()
}
