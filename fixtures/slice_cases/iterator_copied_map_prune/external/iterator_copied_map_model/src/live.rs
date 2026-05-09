#[derive(Clone, Copy)]
pub struct IteratorCopiedMapPayload {
    value: usize,
}

impl IteratorCopiedMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().len(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-copied-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-copied-map:{}", self.value)
    }
}

pub fn selected_iterator_copied_map(raw: &str) -> String {
    let items = [IteratorCopiedMapPayload::new(raw)];
    items
        .iter()
        .copied()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| format!("iterator-copied-map:missing"))
}

pub fn dead_live_iterator_copied_map(raw: &str) -> String {
    IteratorCopiedMapPayload::new(raw).dead_method()
}
