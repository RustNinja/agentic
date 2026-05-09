pub fn selected_mem_swap_map_report(raw: &str) -> String {
    mem_swap_map_model::selected_mem_swap_map(raw)
}

pub fn dead_live_mem_swap_map_report(raw: &str) -> String {
    format!("dead-live-mem-swap-map:{raw}")
}
