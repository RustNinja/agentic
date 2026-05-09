#[derive(Clone)]
pub struct IteratorFilterPredicateMapPayload {
    value: String,
}

impl IteratorFilterPredicateMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-filter-predicate-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("iterator-filter-predicate-map:{}", self.value)
    }

    pub fn is_live(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-filter-predicate-map:{}", self.value)
    }
}

pub fn selected_iterator_filter_predicate_map(raw: &str) -> String {
    let items = vec![IteratorFilterPredicateMapPayload::new(raw)];
    items
        .iter()
        .filter(|payload| payload.is_live())
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| format!("iterator-filter-predicate-map:missing"))
}

pub fn dead_live_iterator_filter_predicate_map(raw: &str) -> String {
    let mut payload = IteratorFilterPredicateMapPayload::new(raw);
    payload.bump_and_render()
}
