pub fn selected_cfg_report(raw: &str) -> String {
    cfg_model::selected_cfg_record(raw).render()
}

pub fn dead_live_cfg_report(raw: &str) -> String {
    format!("dead-live-cfg:{raw}")
}

