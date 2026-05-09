pub struct DeadIteratorByRefTakeMapItem {
    value: String,
}

impl DeadIteratorByRefTakeMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-iterator-by-ref-take-map:{}", self.value)
    }
}

pub fn dead_iterator_by_ref_take_map(raw: &str) -> String {
    DeadIteratorByRefTakeMapItem::new(raw).render()
}
