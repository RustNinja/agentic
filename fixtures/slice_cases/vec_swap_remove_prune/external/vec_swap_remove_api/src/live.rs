pub fn selected_vec_swap_remove_report(raw: &str) -> String {
    vec_swap_remove_model::selected_vec_swap_remove(raw)
}

pub fn dead_live_vec_swap_remove_report(raw: &str) -> String {
    format!("dead-vec-swap-remove-live-report:{raw}")
}
