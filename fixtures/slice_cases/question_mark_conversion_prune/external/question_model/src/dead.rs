pub struct DeadQuestion {
    value: String,
}

pub fn dead_question(raw: &str) -> String {
    format!("dead-question-model:{}", DeadQuestion { value: raw.into() }.value)
}
