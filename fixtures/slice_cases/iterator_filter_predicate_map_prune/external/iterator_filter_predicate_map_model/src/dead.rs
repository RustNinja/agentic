pub struct DeadIteratorFilterPredicateMapItem {
    value: String,
}

impl DeadIteratorFilterPredicateMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-iterator-filter-predicate-map:{}", self.value)
    }
}

pub fn dead_iterator_filter_predicate_map(raw: &str) -> String {
    DeadIteratorFilterPredicateMapItem::new(raw).render()
}
