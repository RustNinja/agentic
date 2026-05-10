pub struct DeadBoxLeakMutMapItem {
    value: String,
}

impl DeadBoxLeakMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-box-leak-mut-map:{}", self.value)
    }
}

pub fn dead_box_leak_mut_map(raw: &str) -> String {
    DeadBoxLeakMutMapItem::new(raw).dead_method()
}
