pub struct DeadWireError {
    raw: String,
}

pub fn dead_error(raw: &str) -> String {
    format!("dead-error-model:{}", DeadWireError { raw: raw.into() }.raw)
}
