pub struct DeadProjection {
    value: String,
}

pub fn dead_projection(raw: &str) -> String {
    format!(
        "dead-projection-model:{}",
        DeadProjection { value: raw.into() }.value
    )
}
