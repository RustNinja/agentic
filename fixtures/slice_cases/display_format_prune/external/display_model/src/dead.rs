pub struct DeadDisplay {
    value: String,
}

pub fn dead_display(raw: &str) -> String {
    format!("dead-display-model:{}", DeadDisplay { value: raw.into() }.value)
}
