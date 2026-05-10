pub struct DeadMaybeuninitWriteMapItem {
    value: String,
}

impl DeadMaybeuninitWriteMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-maybeuninit-write-map:{}", self.value)
    }
}

pub fn dead_maybeuninit_write_map(raw: &str) -> String {
    DeadMaybeuninitWriteMapItem::new(raw).dead_method()
}
