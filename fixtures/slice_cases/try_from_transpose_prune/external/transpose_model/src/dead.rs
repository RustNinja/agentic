pub struct DeadTranspose {
    value: String,
}

pub fn dead_transpose(raw: &str) -> String {
    format!(
        "dead-transpose-model:{}",
        DeadTranspose { value: raw.into() }.value
    )
}
