pub struct DeadVecdequeAsMutSlicesIterItem {
    value: String,
}

impl DeadVecdequeAsMutSlicesIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vecdeque-as-mut-slices-iter:{}", self.value)
    }
}

pub fn dead_vecdeque_as_mut_slices_iter(raw: &str) -> String {
    DeadVecdequeAsMutSlicesIterItem::new(raw).render()
}
