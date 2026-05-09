pub fn selected_mem_replace_map_report(raw: &str) -> String {
    mem_replace_map_model::selected_mem_replace_map(raw)
}

pub fn dead_live_mem_replace_map_report(raw: &str) -> String {
    format!("dead-live-mem-replace-map:{raw}")
}
