pub struct DeadParse {
    value: String,
}

pub fn dead_parse(raw: &str) -> String {
    format!("dead-parse-model:{}", DeadParse { value: raw.into() }.value)
}
