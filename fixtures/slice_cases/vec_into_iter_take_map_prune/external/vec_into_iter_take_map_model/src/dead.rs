pub struct DeadVecIntoIterTakeMapItem {
    value: String,
}

impl DeadVecIntoIterTakeMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-into-iter-take-map:{}", self.value)
    }
}

pub fn dead_vec_into_iter_take_map(raw: &str) -> String {
    DeadVecIntoIterTakeMapItem::new(raw).render()
}
