use std::sync::mpsc::{self, Receiver, SyncSender};
pub struct SyncMpscRecvMapPayload {
    value: String,
}

impl SyncMpscRecvMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("sync-mpsc-recv-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("sync-mpsc-recv-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-sync-mpsc-recv-map:{}", self.value)
    }
}

pub fn selected_sync_mpsc_recv_map(raw: &str) -> String {
    let (tx, rx): (SyncSender<SyncMpscRecvMapPayload>, Receiver<SyncMpscRecvMapPayload>) = mpsc::sync_channel(1);
    tx.send(SyncMpscRecvMapPayload::new(raw)).ok();
    rx.recv()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_sync_mpsc_recv_map(raw: &str) -> String {
    SyncMpscRecvMapPayload::new(raw).unused_label()
}
