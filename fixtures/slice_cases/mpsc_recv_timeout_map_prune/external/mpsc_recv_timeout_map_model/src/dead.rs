pub struct DeadMpscRecvTimeoutMapItem;

pub struct DeadMpscRecvTimeoutMapPayload {
    value: String,
}

impl DeadMpscRecvTimeoutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mpsc-recv-timeout-map:{}", self.value)
    }
}

pub fn dead_mpsc_recv_timeout_map(raw: &str) -> String {
    DeadMpscRecvTimeoutMapPayload::new(raw).dead_method()
}
