use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct SliceSelectNthUnstableByItem {
    value: String,
}

impl SliceSelectNthUnstableByItem {
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
        format!("slice-select-nth-unstable-by:{}", self.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-slice-select-nth-unstable-by:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-select-nth-unstable-by:{}", self.value)
    }
}

fn slice_select_nth_unstable_by_entries(raw: &str) -> Vec<SliceSelectNthUnstableByItem> {
    vec![SliceSelectNthUnstableByItem::live(), SliceSelectNthUnstableByItem::other(), SliceSelectNthUnstableByItem::new(raw)]
}

pub fn selected_slice_select_nth_unstable_by(raw: &str) -> String {
    let mut items = slice_select_nth_unstable_by_entries(raw);
    let _ = items.select_nth_unstable_by(1, |left, right| left.compare(right));
    items
        .iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_slice_select_nth_unstable_by(raw: &str) -> String {
    SliceSelectNthUnstableByItem::new(raw).dead_method()
}
