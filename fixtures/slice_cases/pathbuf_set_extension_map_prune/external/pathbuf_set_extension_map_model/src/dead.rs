pub struct DeadPathbufSetExtensionMapItem {
    value: String,
}

impl DeadPathbufSetExtensionMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-pathbuf-set-extension-map:{}", self.value)
    }
}

pub fn dead_pathbuf_set_extension_map(raw: &str) -> String {
    DeadPathbufSetExtensionMapItem::new(raw).dead_method()
}
