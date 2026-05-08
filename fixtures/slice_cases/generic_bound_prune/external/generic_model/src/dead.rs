pub struct DeadGeneric {
    value: String,
}

pub fn dead_generic(raw: &str) -> String {
    format!("dead-generic-model:{}", DeadGeneric { value: raw.into() }.value)
}
