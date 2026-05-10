pub struct DeadMpscRecvUnwrapMapItem;

pub struct DeadMpscRecvUnwrapMapPayload {
    value: String,
}

impl DeadMpscRecvUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mpsc-recv-unwrap-map:{}", self.value)
    }
}

pub fn dead_mpsc_recv_unwrap_map(raw: &str) -> String {
    DeadMpscRecvUnwrapMapPayload::new(raw).dead_method()
}
