use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct VecAsSliceIterItem {
    value: String,
}

impl VecAsSliceIterItem {
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

    pub fn render_label(&self) -> String {
        format!("vec-as-slice-iter:{}", self.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-vec-as-slice-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-as-slice-iter:{}", self.value)
    }
}

fn vec_as_slice_iter_entries(raw: &str) -> Vec<VecAsSliceIterItem> {
    vec![VecAsSliceIterItem::live(), VecAsSliceIterItem::other(), VecAsSliceIterItem::new(raw)]
}

pub fn selected_vec_as_slice_iter(raw: &str) -> String {
    let entries = vec_as_slice_iter_entries(raw);
    entries
        .as_slice()
        .iter()
        .find(|item| item.is_live())
        .map(|item| item.render_label())
        .unwrap_or_default()
}

pub fn dead_live_vec_as_slice_iter(raw: &str) -> String {
    VecAsSliceIterItem::new(raw).dead_method()
}
