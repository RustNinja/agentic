use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct VecAsSliceChunksItem {
    value: String,
}

impl VecAsSliceChunksItem {
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
        format!("vec-as-slice-chunks:{}", self.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-vec-as-slice-chunks:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-as-slice-chunks:{}", self.value)
    }
}

fn vec_as_slice_chunks_entries(raw: &str) -> Vec<VecAsSliceChunksItem> {
    vec![VecAsSliceChunksItem::live(), VecAsSliceChunksItem::other(), VecAsSliceChunksItem::new(raw)]
}

pub fn selected_vec_as_slice_chunks(raw: &str) -> String {
    let entries = vec_as_slice_chunks_entries(raw);
    entries
        .as_slice()
        .chunks(2)
        .map(|group| group.iter().map(|item| item.render_label()).collect::<Vec<_>>().join("+"))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_vec_as_slice_chunks(raw: &str) -> String {
    VecAsSliceChunksItem::new(raw).dead_method()
}
