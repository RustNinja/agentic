pub struct DeadConstRecord {
    raw: String,
}

pub fn dead_const(raw: &str) -> String {
    format!("dead-const-record:{}", DeadConstRecord { raw: raw.into() }.raw)
}
