#[derive(Clone, Debug)]
pub struct SliceChunksItem {
    value: String,
}

impl SliceChunksItem {
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
        format!("slice-chunks:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-chunks:{}", self.value)
    }
}

fn slice_chunks_entries(raw: &str) -> Vec<SliceChunksItem> {
    vec![SliceChunksItem::live(), SliceChunksItem::breakpoint(), SliceChunksItem::new(raw)]
}

pub fn selected_slice_chunks(raw: &str) -> String {
    let entries = slice_chunks_entries(raw);
    entries
        .chunks(2)
        .map(|chunk| chunk.first().map(|item| item.render_label()).unwrap_or_else(|| "slice-chunks:empty".to_string()))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_slice_chunks(raw: &str) -> String {
    SliceChunksItem::new(raw).dead_method()
}
