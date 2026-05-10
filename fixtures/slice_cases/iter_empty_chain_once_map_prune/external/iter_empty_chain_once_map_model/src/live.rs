use std::iter;
#[derive(Clone)]
pub struct IterEmptyChainOnceMapPayload {
    value: String,
}

impl IterEmptyChainOnceMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iter-empty-chain-once-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("iter-empty-chain-once-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iter-empty-chain-once-map:{}", self.value)
    }
}

pub fn selected_iter_empty_chain_once_map(raw: &str) -> String {
    iter::empty::<IterEmptyChainOnceMapPayload>()
        .chain(iter::once(IterEmptyChainOnceMapPayload::new(raw)))
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_iter_empty_chain_once_map(raw: &str) -> String {
    IterEmptyChainOnceMapPayload::new(raw).unused_label()
}
