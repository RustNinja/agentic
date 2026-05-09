pub struct DeadResultIterMutMapItem {
    value: String,
}

impl DeadResultIterMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-iter-mut-map:{}", self.value)
    }
}

pub fn dead_result_iter_mut_map(raw: &str) -> String {
    DeadResultIterMutMapItem::new(raw).render()
}
