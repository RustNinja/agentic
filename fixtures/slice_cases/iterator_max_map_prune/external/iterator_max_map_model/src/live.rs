#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct IteratorMaxMapPayload {
    value: String,
}

impl IteratorMaxMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-max-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("iterator-max-map:{}", self.value)
    }

    pub fn is_match(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-max-map:{}", self.value)
    }
}

fn iterator_max_map_items(raw: &str) -> Vec<IteratorMaxMapPayload> {
    vec![IteratorMaxMapPayload::new("head"), IteratorMaxMapPayload::new(raw)]
}

pub fn selected_iterator_max_map(raw: &str) -> String {
    iterator_max_map_items(raw)
        .into_iter()
        .max()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "iterator-max-map:missing".to_string())
}

pub fn dead_live_iterator_max_map(raw: &str) -> String {
    IteratorMaxMapPayload::new(raw).unused_label()
}
