pub fn selected_option_iter_mut_map_report(raw: &str) -> String {
    option_iter_mut_map_model::selected_option_iter_mut_map(raw)
}

pub fn dead_live_option_iter_mut_map_report(raw: &str) -> String {
    format!("dead-live-option-iter-mut-map:{raw}")
}
