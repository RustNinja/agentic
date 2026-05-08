pub struct DeadFreeClosure {
    value: String,
}

pub fn dead_free_closure(raw: &str) -> String {
    format!(
        "dead-free-closure-model:{}",
        DeadFreeClosure { value: raw.into() }.value
    )
}
