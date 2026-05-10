pub struct IteratorPeekablePeekMutPayload {
    value: String,
}

impl IteratorPeekablePeekMutPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-peekable-peek-mut:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("iterator-peekable-peek-mut:{}", self.value)
    }

    pub fn is_match(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-peekable-peek-mut:{}", self.value)
    }
}

fn iterator_peekable_peek_mut_items(raw: &str) -> Vec<IteratorPeekablePeekMutPayload> {
    vec![IteratorPeekablePeekMutPayload::new(raw), IteratorPeekablePeekMutPayload::new("tail")]
}

pub fn selected_iterator_peekable_peek_mut(raw: &str) -> String {
    let mut items = iterator_peekable_peek_mut_items(raw);
    let mut iter = items.iter_mut().peekable();
    iter.peek_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_else(|| "iterator-peekable-peek-mut:missing".to_string())
}

pub fn dead_live_iterator_peekable_peek_mut(raw: &str) -> String {
    IteratorPeekablePeekMutPayload::new(raw).unused_label()
}
