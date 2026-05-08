pub struct DeadClosureReturn {
    value: String,
}

pub fn dead_closure_return(raw: &str) -> String {
    format!(
        "dead-closure-return-model:{}",
        DeadClosureReturn { value: raw.into() }.value
    )
}
