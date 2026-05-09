#[derive(Clone, Debug)]
pub struct SliceChunksExactItem {
    value: String,
}

impl SliceChunksExactItem {
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
        format!("slice-chunks-exact:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-chunks-exact:{}", self.value)
    }
}

fn slice_chunks_exact_entries(raw: &str) -> Vec<SliceChunksExactItem> {
    vec![SliceChunksExactItem::live(), SliceChunksExactItem::breakpoint(), SliceChunksExactItem::new(raw)]
}

pub fn selected_slice_chunks_exact(raw: &str) -> String {
    let entries = slice_chunks_exact_entries(raw);
    entries
        .chunks_exact(2)
        .map(|chunk| chunk.iter().map(|item| item.render_label()).collect::<Vec<_>>().join("+"))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_slice_chunks_exact(raw: &str) -> String {
    SliceChunksExactItem::new(raw).dead_method()
}
