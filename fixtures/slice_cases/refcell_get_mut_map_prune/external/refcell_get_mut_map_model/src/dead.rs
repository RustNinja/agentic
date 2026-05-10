pub struct DeadRefcellGetMutMapItem {
    value: String,
}

impl DeadRefcellGetMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-refcell-get-mut-map:{}", self.value)
    }
}

pub fn dead_refcell_get_mut_map(raw: &str) -> String {
    DeadRefcellGetMutMapItem::new(raw).dead_method()
}
