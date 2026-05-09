#[derive(Clone, Debug)]
pub struct SliceRsplitItem {
    value: String,
}

impl SliceRsplitItem {
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
        format!("slice-rsplit:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rsplit:{}", self.value)
    }
}

fn slice_rsplit_entries(raw: &str) -> Vec<SliceRsplitItem> {
    vec![SliceRsplitItem::live(), SliceRsplitItem::breakpoint(), SliceRsplitItem::new(raw)]
}

pub fn selected_slice_rsplit(raw: &str) -> String {
    let entries = slice_rsplit_entries(raw);
    entries
        .rsplit(|item| item.is_break())
        .map(|group| group.first().map(|item| item.render_label()).unwrap_or_else(|| "slice-rsplit:empty".to_string()))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_slice_rsplit(raw: &str) -> String {
    SliceRsplitItem::new(raw).dead_method()
}
