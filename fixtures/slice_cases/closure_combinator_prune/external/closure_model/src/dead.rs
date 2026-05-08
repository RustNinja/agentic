pub struct DeadClosure {
    raw: String,
}

pub fn dead_closure(raw: &str) -> String {
    format!("dead-closure:{}", DeadClosure { raw: raw.into() }.raw)
}
