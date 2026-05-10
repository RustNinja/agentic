pub struct DeadVecdequeIterNextBackMapItem;

pub struct DeadVecdequeIterNextBackMapPayload {
    value: String,
}

impl DeadVecdequeIterNextBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-iter-next-back-map:{}", self.value)
    }
}

pub fn dead_vecdeque_iter_next_back_map(raw: &str) -> String {
    DeadVecdequeIterNextBackMapPayload::new(raw).dead_method()
}
