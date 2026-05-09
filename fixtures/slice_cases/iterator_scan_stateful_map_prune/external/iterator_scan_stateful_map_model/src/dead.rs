pub struct DeadIteratorScanStatefulMapItem {
    value: String,
}

impl DeadIteratorScanStatefulMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-scan-stateful-map:{}", self.value)
    }
}

pub fn dead_iterator_scan_stateful_map(raw: &str) -> String {
    DeadIteratorScanStatefulMapItem::new(raw).dead_method()
}
