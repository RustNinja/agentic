pub struct DeadBoxIntoRawFromRawMapItem;

pub struct DeadBoxIntoRawFromRawMapPayload {
    value: String,
}

impl DeadBoxIntoRawFromRawMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-box-into-raw-from-raw-map:{}", self.value)
    }
}

pub fn dead_box_into_raw_from_raw_map(raw: &str) -> String {
    DeadBoxIntoRawFromRawMapPayload::new(raw).dead_method()
}
