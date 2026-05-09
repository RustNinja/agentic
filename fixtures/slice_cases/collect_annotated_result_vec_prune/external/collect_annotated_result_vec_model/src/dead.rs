pub struct DeadCollectAnnotatedResultVecItem {
    value: String,
}

impl DeadCollectAnnotatedResultVecItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn render(&self) -> String {
        format!("dead-model-collect-annotated-result-vec:{}", self.value)
    }
}

pub fn dead_collect_annotated_result_vec_model(raw: &str) -> String {
    DeadCollectAnnotatedResultVecItem::new(raw).render()
}
