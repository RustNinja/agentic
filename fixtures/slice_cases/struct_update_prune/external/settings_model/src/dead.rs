pub struct DeadSettings {
    raw: String,
}

pub fn dead_settings(raw: &str) -> String {
    format!("dead-settings-model:{}", DeadSettings { raw: raw.into() }.raw)
}
