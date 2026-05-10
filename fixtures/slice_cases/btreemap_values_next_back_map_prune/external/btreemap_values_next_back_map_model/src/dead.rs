pub struct DeadBtreemapValuesNextBackMapItem;

pub struct DeadBtreemapValuesNextBackMapPayload {
    value: String,
}

impl DeadBtreemapValuesNextBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-values-next-back-map:{}", self.value)
    }
}

pub fn dead_btreemap_values_next_back_map(raw: &str) -> String {
    DeadBtreemapValuesNextBackMapPayload::new(raw).dead_method()
}
