pub struct DeadCfgRecord {
    value: String,
}

pub fn dead_cfg_record(raw: &str) -> String {
    format!("dead-cfg-model:{}", DeadCfgRecord { value: raw.into() }.value)
}

