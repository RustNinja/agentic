pub struct DeadOptionZipTupleRenderItem {
    value: String,
}

impl DeadOptionZipTupleRenderItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-zip-tuple-render:{}", self.value)
    }
}

pub fn dead_option_zip_tuple_render(raw: &str) -> String {
    DeadOptionZipTupleRenderItem::new(raw).dead_method()
}
