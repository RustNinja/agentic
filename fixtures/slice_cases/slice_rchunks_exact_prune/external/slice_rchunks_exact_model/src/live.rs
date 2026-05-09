#[derive(Clone, Debug)]
pub struct SliceRchunksExactItem {
    value: String,
}

impl SliceRchunksExactItem {
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
        format!("slice-rchunks-exact:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rchunks-exact:{}", self.value)
    }
}

fn slice_rchunks_exact_entries(raw: &str) -> Vec<SliceRchunksExactItem> {
    vec![SliceRchunksExactItem::live(), SliceRchunksExactItem::breakpoint(), SliceRchunksExactItem::new(raw)]
}

pub fn selected_slice_rchunks_exact(raw: &str) -> String {
    let entries = slice_rchunks_exact_entries(raw);
    entries
        .rchunks_exact(2)
        .map(|chunk| chunk.iter().map(|item| item.render_label()).collect::<Vec<_>>().join("+"))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_slice_rchunks_exact(raw: &str) -> String {
    SliceRchunksExactItem::new(raw).dead_method()
}
