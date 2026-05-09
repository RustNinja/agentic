pub fn selected_vec_swap_iter_report(raw: &str) -> String {
    vec_swap_iter_model::selected_vec_swap_iter(raw)
}

pub fn dead_live_vec_swap_iter_report(raw: &str) -> String {
    format!("dead-live-vec-swap-iter:{raw}")
}
