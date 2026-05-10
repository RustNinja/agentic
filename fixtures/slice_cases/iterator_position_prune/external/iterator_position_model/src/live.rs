pub struct IteratorPositionPayload {
    value: String,
}

impl IteratorPositionPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-position:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("iterator-position:{}", self.value)
    }

    pub fn is_match(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-position:{}", self.value)
    }
}

fn iterator_position_items(raw: &str) -> Vec<IteratorPositionPayload> {
    vec![IteratorPositionPayload::new("head"), IteratorPositionPayload::new(raw)]
}

pub fn selected_iterator_position(raw: &str) -> String {
    iterator_position_items(raw)
        .iter()
        .position(|payload| payload.is_match())
        .map(|index| format!("iterator-position:{index}"))
        .unwrap_or_else(|| "iterator-position:missing".to_string())
}

pub fn dead_live_iterator_position(raw: &str) -> String {
    IteratorPositionPayload::new(raw).unused_label()
}
