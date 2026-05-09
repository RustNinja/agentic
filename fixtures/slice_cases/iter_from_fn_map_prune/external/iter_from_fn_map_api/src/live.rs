pub fn selected_iter_from_fn_map_report(raw: &str) -> String {
    iter_from_fn_map_model::selected_iter_from_fn_map(raw)
}

pub fn dead_live_iter_from_fn_map_report(raw: &str) -> String {
    format!("dead-live-iter-from-fn-map:{raw}")
}
