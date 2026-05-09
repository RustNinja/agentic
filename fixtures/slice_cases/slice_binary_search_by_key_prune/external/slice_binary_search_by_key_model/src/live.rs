use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct SliceBinarySearchByKeyItem {
    value: String,
}

impl SliceBinarySearchByKeyItem {
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
        format!("slice-binary-search-by-key:{}", self.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-slice-binary-search-by-key:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-binary-search-by-key:{}", self.value)
    }
}

fn slice_binary_search_by_key_entries(raw: &str) -> Vec<SliceBinarySearchByKeyItem> {
    vec![SliceBinarySearchByKeyItem::live(), SliceBinarySearchByKeyItem::other(), SliceBinarySearchByKeyItem::new(raw)]
}

pub fn selected_slice_binary_search_by_key(raw: &str) -> String {
    let mut items = slice_binary_search_by_key_entries(raw);
    items.sort_by_key(|item| item.sort_key());
    let needle = raw.len();
    items
        .binary_search_by_key(&needle, |item| item.sort_key())
        .ok()
        .and_then(|index| items.get(index))
        .map(|item| item.render_label())
        .unwrap_or_else(|| String::from("missing"))
}

pub fn dead_live_slice_binary_search_by_key(raw: &str) -> String {
    SliceBinarySearchByKeyItem::new(raw).dead_method()
}
