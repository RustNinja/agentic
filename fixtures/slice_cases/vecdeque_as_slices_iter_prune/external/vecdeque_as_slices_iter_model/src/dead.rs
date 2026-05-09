pub struct DeadVecdequeAsSlicesIterItem {
    value: String,
}

impl DeadVecdequeAsSlicesIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vecdeque-as-slices-iter:{}", self.value)
    }
}

pub fn dead_vecdeque_as_slices_iter(raw: &str) -> String {
    DeadVecdequeAsSlicesIterItem::new(raw).render()
}
