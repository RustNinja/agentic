pub struct IteratorPeekableNextIfPayload {
    value: String,
}

impl IteratorPeekableNextIfPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-peekable-next-if:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("iterator-peekable-next-if:{}", self.value)
    }

    pub fn is_match(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-peekable-next-if:{}", self.value)
    }
}

fn iterator_peekable_next_if_items(raw: &str) -> Vec<IteratorPeekableNextIfPayload> {
    vec![IteratorPeekableNextIfPayload::new(raw), IteratorPeekableNextIfPayload::new("tail")]
}

pub fn selected_iterator_peekable_next_if(raw: &str) -> String {
    let mut iter = iterator_peekable_next_if_items(raw).into_iter().peekable();
    iter.next_if(|payload| payload.is_match())
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "iterator-peekable-next-if:missing".to_string())
}

pub fn dead_live_iterator_peekable_next_if(raw: &str) -> String {
    IteratorPeekableNextIfPayload::new(raw).unused_label()
}
