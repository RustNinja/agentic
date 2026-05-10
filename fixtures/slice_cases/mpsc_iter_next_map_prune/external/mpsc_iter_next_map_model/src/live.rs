use std::sync::mpsc::{self, Receiver, Sender};
pub struct MpscIterNextMapPayload {
    value: String,
}

impl MpscIterNextMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mpsc-iter-next-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("mpsc-iter-next-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-mpsc-iter-next-map:{}", self.value)
    }
}

pub fn selected_mpsc_iter_next_map(raw: &str) -> String {
    let (tx, rx): (Sender<MpscIterNextMapPayload>, Receiver<MpscIterNextMapPayload>) = mpsc::channel();
    tx.send(MpscIterNextMapPayload::new(raw)).ok();
    drop(tx);
    rx.iter()
        .next()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_mpsc_iter_next_map(raw: &str) -> String {
    MpscIterNextMapPayload::new(raw).unused_label()
}
