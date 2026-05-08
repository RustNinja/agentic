pub struct DeadIterator {
    value: String,
}

pub fn dead_iterator(raw: &str) -> String {
    format!("dead-iterator-model:{}", DeadIterator { value: raw.into() }.value)
}
