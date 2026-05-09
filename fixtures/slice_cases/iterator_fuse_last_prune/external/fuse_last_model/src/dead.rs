pub struct DeadFuseLastItem {
    value: String,
}

impl DeadFuseLastItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-fuse-last:{}", self.value)
    }
}

pub fn dead_fuse_last(raw: &str) -> String {
    DeadFuseLastItem::new(raw).dead_method()
}
