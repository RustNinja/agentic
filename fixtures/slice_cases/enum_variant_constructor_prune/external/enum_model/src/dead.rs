pub struct DeadEnum {
    label: String,
}

pub fn dead_enum(raw: &str) -> String {
    format!("dead-enum-model:{}", DeadEnum { label: raw.into() }.label)
}
