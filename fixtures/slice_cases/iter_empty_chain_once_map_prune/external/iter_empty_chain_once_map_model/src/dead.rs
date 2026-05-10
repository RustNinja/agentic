pub struct DeadIterEmptyChainOnceMapItem;

pub struct DeadIterEmptyChainOnceMapPayload {
    value: String,
}

impl DeadIterEmptyChainOnceMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iter-empty-chain-once-map:{}", self.value)
    }
}

pub fn dead_iter_empty_chain_once_map(raw: &str) -> String {
    DeadIterEmptyChainOnceMapPayload::new(raw).dead_method()
}
