#[derive(Clone, Debug)]
pub struct SliceSplitInclusiveItem {
    value: String,
}

impl SliceSplitInclusiveItem {
    pub fn live() -> Self {
        Self::new("live")
    }

    pub fn breakpoint() -> Self {
        Self::new("break")
    }

    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn is_break(&self) -> bool {
        self.value == "break"
    }

    pub fn bump(&mut self) -> &mut Self {
        self.value.push_str("-bumped");
        self
    }

    pub fn render_label(&self) -> String {
        format!("slice-split-inclusive:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split-inclusive:{}", self.value)
    }
}

fn slice_split_inclusive_entries(raw: &str) -> Vec<SliceSplitInclusiveItem> {
    vec![SliceSplitInclusiveItem::live(), SliceSplitInclusiveItem::breakpoint(), SliceSplitInclusiveItem::new(raw)]
}

pub fn selected_slice_split_inclusive(raw: &str) -> String {
    let entries = slice_split_inclusive_entries(raw);
    entries
        .split_inclusive(|item| item.is_break())
        .map(|group| group.last().map(|item| item.render_label()).unwrap_or_else(|| "slice-split-inclusive:empty".to_string()))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_slice_split_inclusive(raw: &str) -> String {
    SliceSplitInclusiveItem::new(raw).dead_method()
}
