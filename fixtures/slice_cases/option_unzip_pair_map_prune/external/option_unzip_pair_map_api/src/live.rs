pub fn selected_option_unzip_pair_map_report(raw: &str) -> String {
    option_unzip_pair_map_model::selected_option_unzip_pair_map(raw)
}

pub fn dead_live_option_unzip_pair_map_report(raw: &str) -> String {
    format!("dead-live-option-unzip-pair-map:{raw}")
}
