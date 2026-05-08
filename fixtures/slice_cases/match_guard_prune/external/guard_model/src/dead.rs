pub struct DeadGuard {
    value: String,
}

pub fn dead_guard(raw: &str) -> String {
    format!("dead-guard-model:{}", DeadGuard { value: raw.into() }.value)
}
