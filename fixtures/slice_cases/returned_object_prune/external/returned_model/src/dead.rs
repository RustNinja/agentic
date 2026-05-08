pub struct DeadReturned {
    value: String,
}

pub fn dead_returned(raw: &str) -> String {
    format!(
        "dead-returned-model:{}",
        DeadReturned { value: raw.into() }.value
    )
}
