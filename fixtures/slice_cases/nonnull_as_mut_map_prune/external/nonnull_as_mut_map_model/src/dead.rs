pub struct DeadNonnullAsMutMapItem {
    value: String,
}

impl DeadNonnullAsMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-nonnull-as-mut-map:{}", self.value)
    }
}

pub fn dead_nonnull_as_mut_map(raw: &str) -> String {
    DeadNonnullAsMutMapItem::new(raw).dead_method()
}
