pub fn selected_mpsc_iter_next_map_report(raw: &str) -> String {
    mpsc_iter_next_map_model::selected_mpsc_iter_next_map(raw)
}

pub fn dead_live_mpsc_iter_next_map_report(raw: &str) -> String {
    format!("dead-live-mpsc-iter-next-map-report:{raw}")
}
