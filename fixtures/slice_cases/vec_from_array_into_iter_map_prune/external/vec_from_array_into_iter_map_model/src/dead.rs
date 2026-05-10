pub struct DeadVecFromArrayIntoIterMapItem;

pub struct DeadVecFromArrayIntoIterMapPayload {
    value: String,
}

impl DeadVecFromArrayIntoIterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-from-array-into-iter-map:{}", self.value)
    }
}

pub fn dead_vec_from_array_into_iter_map(raw: &str) -> String {
    DeadVecFromArrayIntoIterMapPayload::new(raw).dead_method()
}
