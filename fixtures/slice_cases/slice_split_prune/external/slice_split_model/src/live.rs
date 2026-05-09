#[derive(Clone, Debug)]
pub struct SliceSplitItem {
    value: String,
}

impl SliceSplitItem {
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
        format!("slice-split:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split:{}", self.value)
    }
}

fn slice_split_entries(raw: &str) -> Vec<SliceSplitItem> {
    vec![SliceSplitItem::live(), SliceSplitItem::breakpoint(), SliceSplitItem::new(raw)]
}

pub fn selected_slice_split(raw: &str) -> String {
    let entries = slice_split_entries(raw);
    entries
        .split(|item| item.is_break())
        .map(|group| group.first().map(|item| item.render_label()).unwrap_or_else(|| "slice-split:empty".to_string()))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_slice_split(raw: &str) -> String {
    SliceSplitItem::new(raw).dead_method()
}
