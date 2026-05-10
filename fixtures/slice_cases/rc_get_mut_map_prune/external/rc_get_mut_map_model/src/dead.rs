pub struct DeadRcGetMutMapItem {
    value: String,
}

impl DeadRcGetMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rc-get-mut-map:{}", self.value)
    }
}

pub fn dead_rc_get_mut_map(raw: &str) -> String {
    DeadRcGetMutMapItem::new(raw).dead_method()
}
