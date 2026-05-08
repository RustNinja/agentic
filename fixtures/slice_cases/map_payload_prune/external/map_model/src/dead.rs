pub struct DeadMap {
    value: String,
}

pub fn dead_map(raw: &str) -> String {
    format!("dead-map-model:{}", DeadMap { value: raw.into() }.value)
}
