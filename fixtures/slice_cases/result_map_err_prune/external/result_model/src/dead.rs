pub struct DeadResult {
    value: String,
}

pub fn dead_result(raw: &str) -> String {
    format!("dead-result-model:{}", DeadResult { value: raw.into() }.value)
}
