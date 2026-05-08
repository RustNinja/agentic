pub struct DeadMacroRecord {
    raw: String,
}

pub fn dead_macro(raw: &str) -> String {
    format!("dead-macro-record:{}", DeadMacroRecord { raw: raw.into() }.raw)
}
