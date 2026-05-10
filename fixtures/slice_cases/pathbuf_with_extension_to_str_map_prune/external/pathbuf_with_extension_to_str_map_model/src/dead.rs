pub struct DeadPathbufWithExtensionToStrMapItem {
    value: String,
}

impl DeadPathbufWithExtensionToStrMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-pathbuf-with-extension-to-str-map:{}", self.value)
    }
}

pub fn dead_pathbuf_with_extension_to_str_map(raw: &str) -> String {
    DeadPathbufWithExtensionToStrMapItem::new(raw).dead_method()
}
