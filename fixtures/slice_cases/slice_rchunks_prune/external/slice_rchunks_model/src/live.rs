#[derive(Clone, Debug)]
pub struct SliceRchunksItem {
    value: String,
}

impl SliceRchunksItem {
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
        format!("slice-rchunks:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rchunks:{}", self.value)
    }
}

fn slice_rchunks_entries(raw: &str) -> Vec<SliceRchunksItem> {
    vec![SliceRchunksItem::live(), SliceRchunksItem::breakpoint(), SliceRchunksItem::new(raw)]
}

pub fn selected_slice_rchunks(raw: &str) -> String {
    let entries = slice_rchunks_entries(raw);
    entries
        .rchunks(2)
        .map(|chunk| chunk.last().map(|item| item.render_label()).unwrap_or_else(|| "slice-rchunks:empty".to_string()))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_slice_rchunks(raw: &str) -> String {
    SliceRchunksItem::new(raw).dead_method()
}
