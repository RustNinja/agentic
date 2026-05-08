pub struct DeadOption {
    value: String,
}

pub fn dead_option(raw: &str) -> String {
    format!("dead-option-model:{}", DeadOption { value: raw.into() }.value)
}
