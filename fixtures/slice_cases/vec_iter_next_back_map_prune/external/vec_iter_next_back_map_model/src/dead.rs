pub struct DeadVecIterNextBackMapItem;

pub struct DeadVecIterNextBackMapPayload {
    value: String,
}

impl DeadVecIterNextBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-iter-next-back-map:{}", self.value)
    }
}

pub fn dead_vec_iter_next_back_map(raw: &str) -> String {
    DeadVecIterNextBackMapPayload::new(raw).dead_method()
}
