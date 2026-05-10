pub struct DeadMpscRecvMapItem;

pub struct DeadMpscRecvMapPayload {
    value: String,
}

impl DeadMpscRecvMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mpsc-recv-map:{}", self.value)
    }
}

pub fn dead_mpsc_recv_map(raw: &str) -> String {
    DeadMpscRecvMapPayload::new(raw).dead_method()
}
