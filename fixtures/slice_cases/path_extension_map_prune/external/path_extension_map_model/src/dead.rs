pub struct DeadPathExtensionMapItem {
    value: String,
}

impl DeadPathExtensionMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-path-extension-map:{}", self.value)
    }
}

pub fn dead_path_extension_map(raw: &str) -> String {
    DeadPathExtensionMapItem::new(raw).dead_method()
}
