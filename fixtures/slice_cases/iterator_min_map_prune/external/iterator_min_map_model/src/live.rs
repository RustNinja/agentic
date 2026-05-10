#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct IteratorMinMapPayload {
    value: String,
}

impl IteratorMinMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-min-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("iterator-min-map:{}", self.value)
    }

    pub fn is_match(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-min-map:{}", self.value)
    }
}

fn iterator_min_map_items(raw: &str) -> Vec<IteratorMinMapPayload> {
    vec![IteratorMinMapPayload::new(raw), IteratorMinMapPayload::new("tail")]
}

pub fn selected_iterator_min_map(raw: &str) -> String {
    iterator_min_map_items(raw)
        .into_iter()
        .min()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "iterator-min-map:missing".to_string())
}

pub fn dead_live_iterator_min_map(raw: &str) -> String {
    IteratorMinMapPayload::new(raw).unused_label()
}
