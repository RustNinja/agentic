pub struct DeadVecBoxIterAsRefMapItem {
    value: String,
}

impl DeadVecBoxIterAsRefMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-box-iter-as-ref-map:{}", self.value)
    }
}

pub fn dead_vec_box_iter_as_ref_map(raw: &str) -> String {
    DeadVecBoxIterAsRefMapItem::new(raw).render()
}
