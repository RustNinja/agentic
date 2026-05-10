pub struct DeadHashsetExtendIterMapItem {
    value: String,
}

impl DeadHashsetExtendIterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-extend-iter-map:{}", self.value)
    }
}

pub fn dead_hashset_extend_iter_map(raw: &str) -> String {
    DeadHashsetExtendIterMapItem::new(raw).dead_method()
}
