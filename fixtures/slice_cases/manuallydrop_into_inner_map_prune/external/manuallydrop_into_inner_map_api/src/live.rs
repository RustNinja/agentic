pub fn selected_manuallydrop_into_inner_map_report(raw: &str) -> String {
    manuallydrop_into_inner_map_model::selected_manuallydrop_into_inner_map(raw)
}

pub fn dead_live_manuallydrop_into_inner_map_report(raw: &str) -> String {
    format!("dead-live-manuallydrop-into-inner-map:{raw}")
}
