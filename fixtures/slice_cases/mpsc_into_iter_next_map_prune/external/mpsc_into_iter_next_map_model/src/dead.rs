pub struct DeadMpscIntoIterNextMapItem;

pub struct DeadMpscIntoIterNextMapPayload {
    value: String,
}

impl DeadMpscIntoIterNextMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mpsc-into-iter-next-map:{}", self.value)
    }
}

pub fn dead_mpsc_into_iter_next_map(raw: &str) -> String {
    DeadMpscIntoIterNextMapPayload::new(raw).dead_method()
}
