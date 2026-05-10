pub struct DeadSyncMpscRecvMapItem;

pub struct DeadSyncMpscRecvMapPayload {
    value: String,
}

impl DeadSyncMpscRecvMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-sync-mpsc-recv-map:{}", self.value)
    }
}

pub fn dead_sync_mpsc_recv_map(raw: &str) -> String {
    DeadSyncMpscRecvMapPayload::new(raw).dead_method()
}
