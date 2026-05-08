pub struct DeadIndex {
    value: String,
}

pub fn dead_index(raw: &str) -> String {
    format!("dead-index-model:{}", DeadIndex { value: raw.into() }.value)
}
