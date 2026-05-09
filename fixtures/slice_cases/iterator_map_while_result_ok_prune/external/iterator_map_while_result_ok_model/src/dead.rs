pub struct DeadIteratorMapWhileResultOkItem {
    value: String,
}

impl DeadIteratorMapWhileResultOkItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-map-while-result-ok:{}", self.value)
    }
}

pub fn dead_iterator_map_while_result_ok(raw: &str) -> String {
    DeadIteratorMapWhileResultOkItem::new(raw).dead_method()
}
