pub struct DeadCollectAnnotatedOptionVecItem {
    value: String,
}

impl DeadCollectAnnotatedOptionVecItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn render(&self) -> String {
        format!("dead-model-collect-annotated-option-vec:{}", self.value)
    }
}

pub fn dead_collect_annotated_option_vec_model(raw: &str) -> String {
    DeadCollectAnnotatedOptionVecItem::new(raw).render()
}
