pub struct DeadDeref {
    value: String,
}

pub fn dead_deref(raw: &str) -> String {
    format!("dead-deref-model:{}", DeadDeref { value: raw.into() }.value)
}
