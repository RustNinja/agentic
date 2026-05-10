use std::sync::mpsc::{self, Receiver, Sender};
pub struct MpscRecvMapPayload {
    value: String,
}

impl MpscRecvMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mpsc-recv-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("mpsc-recv-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-mpsc-recv-map:{}", self.value)
    }
}

pub fn selected_mpsc_recv_map(raw: &str) -> String {
    let (tx, rx): (Sender<MpscRecvMapPayload>, Receiver<MpscRecvMapPayload>) = mpsc::channel();
    tx.send(MpscRecvMapPayload::new(raw)).ok();
    rx.recv()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_mpsc_recv_map(raw: &str) -> String {
    MpscRecvMapPayload::new(raw).unused_label()
}
