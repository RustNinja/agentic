pub struct PeekableNthPayload {
    value: String,
}

impl PeekableNthPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn render_label(&self) -> String {
        format!("peekable-nth:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-peekable-nth:{}", self.value)
    }
}

fn peekable_nth_items(raw: &str) -> Vec<PeekableNthPayload> {
    vec![PeekableNthPayload::new(raw), PeekableNthPayload::new("tail")]
}

pub fn selected_peekable_nth(raw: &str) -> String {
    peekable_nth_items(raw)
        .into_iter()
        .peekable()
        .nth(1)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "peekable-nth:missing".to_string())
}

pub fn dead_live_peekable_nth(raw: &str) -> String {
    PeekableNthPayload::new(raw).dead_method()
}
