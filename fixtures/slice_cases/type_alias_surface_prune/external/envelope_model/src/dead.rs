pub struct DeadEnvelope {
    raw: String,
}

pub fn dead_envelope(raw: &str) -> String {
    format!("dead-envelope:{}", DeadEnvelope { raw: raw.into() }.raw)
}
