use std::sync::mpsc::{self, Receiver, Sender};
pub struct MpscTryRecvMapPayload {
    value: String,
}

impl MpscTryRecvMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mpsc-try-recv-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("mpsc-try-recv-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-mpsc-try-recv-map:{}", self.value)
    }
}

pub fn selected_mpsc_try_recv_map(raw: &str) -> String {
    let (tx, rx): (Sender<MpscTryRecvMapPayload>, Receiver<MpscTryRecvMapPayload>) = mpsc::channel();
    tx.send(MpscTryRecvMapPayload::new(raw)).ok();
    rx.try_recv()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_mpsc_try_recv_map(raw: &str) -> String {
    MpscTryRecvMapPayload::new(raw).unused_label()
}
