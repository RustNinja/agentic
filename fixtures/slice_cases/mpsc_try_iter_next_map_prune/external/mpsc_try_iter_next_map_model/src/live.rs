use std::sync::mpsc::{self, Receiver, Sender};
pub struct MpscTryIterNextMapPayload {
    value: String,
}

impl MpscTryIterNextMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mpsc-try-iter-next-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("mpsc-try-iter-next-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-mpsc-try-iter-next-map:{}", self.value)
    }
}

pub fn selected_mpsc_try_iter_next_map(raw: &str) -> String {
    let (tx, rx): (Sender<MpscTryIterNextMapPayload>, Receiver<MpscTryIterNextMapPayload>) = mpsc::channel();
    tx.send(MpscTryIterNextMapPayload::new(raw)).ok();
    rx.try_iter()
        .next()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_mpsc_try_iter_next_map(raw: &str) -> String {
    MpscTryIterNextMapPayload::new(raw).unused_label()
}
