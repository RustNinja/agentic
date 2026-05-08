pub struct DeadWire {
    raw: String,
}

pub fn dead_wire(raw: &str) -> String {
    format!("dead-wire-model:{}", DeadWire { raw: raw.into() }.raw)
}
