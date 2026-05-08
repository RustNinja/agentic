pub struct DeadNewtype {
    value: String,
}

pub fn dead_newtype(raw: &str) -> String {
    format!(
        "dead-newtype-model:{}",
        DeadNewtype { value: raw.into() }.value
    )
}
