use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct SliceBinarySearchByItem {
    value: String,
}

impl SliceBinarySearchByItem {
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
        format!("slice-binary-search-by:{}", self.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-slice-binary-search-by:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-binary-search-by:{}", self.value)
    }
}

fn slice_binary_search_by_entries(raw: &str) -> Vec<SliceBinarySearchByItem> {
    vec![SliceBinarySearchByItem::live(), SliceBinarySearchByItem::other(), SliceBinarySearchByItem::new(raw)]
}

pub fn selected_slice_binary_search_by(raw: &str) -> String {
    let mut items = slice_binary_search_by_entries(raw);
    items.sort_by_key(|item| item.sort_key());
    items
        .binary_search_by(|item| item.compare_key(raw))
        .ok()
        .and_then(|index| items.get(index))
        .map(|item| item.render_label())
        .unwrap_or_else(|| String::from("missing"))
}

pub fn dead_live_slice_binary_search_by(raw: &str) -> String {
    SliceBinarySearchByItem::new(raw).dead_method()
}
