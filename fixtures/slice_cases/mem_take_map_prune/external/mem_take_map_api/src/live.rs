pub fn selected_mem_take_map_report(raw: &str) -> String {
    mem_take_map_model::selected_mem_take_map(raw)
}

pub fn dead_live_mem_take_map_report(raw: &str) -> String {
    format!("dead-live-mem-take-map:{raw}")
}
