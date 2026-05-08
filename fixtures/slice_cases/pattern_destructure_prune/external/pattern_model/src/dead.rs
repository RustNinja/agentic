pub struct DeadPattern {
    raw: String,
}

pub fn dead_pattern(raw: &str) -> String {
    format!("dead-pattern-model:{}", DeadPattern { raw: raw.into() }.raw)
}
