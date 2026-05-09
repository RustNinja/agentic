pub fn selected_collect_option_vec_report(raw: &str) -> String {
    collect_option_vec_model::selected_collect_option_vec(raw)
}

pub fn dead_live_collect_option_vec_report(raw: &str) -> String {
    format!("dead-collect-option-vec-live-report:{raw}")
}
