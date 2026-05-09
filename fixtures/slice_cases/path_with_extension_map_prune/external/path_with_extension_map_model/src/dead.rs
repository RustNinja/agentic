pub struct DeadPathWithExtensionMapItem {
    value: String,
}

impl DeadPathWithExtensionMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-path-with-extension-map:{}", self.value)
    }
}

pub fn dead_path_with_extension_map(raw: &str) -> String {
    DeadPathWithExtensionMapItem::new(raw).dead_method()
}
