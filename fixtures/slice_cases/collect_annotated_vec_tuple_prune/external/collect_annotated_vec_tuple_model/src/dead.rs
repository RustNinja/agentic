pub struct DeadCollectAnnotatedVecTupleItem {
    value: String,
}

impl DeadCollectAnnotatedVecTupleItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn render(&self) -> String {
        format!("dead-model-collect-annotated-vec-tuple:{}", self.value)
    }
}

pub fn dead_collect_annotated_vec_tuple_model(raw: &str) -> String {
    DeadCollectAnnotatedVecTupleItem::new(raw).render()
}
