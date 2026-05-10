pub fn selected_mpsc_try_iter_next_map_report(raw: &str) -> String {
    mpsc_try_iter_next_map_model::selected_mpsc_try_iter_next_map(raw)
}

pub fn dead_live_mpsc_try_iter_next_map_report(raw: &str) -> String {
    format!("dead-live-mpsc-try-iter-next-map-report:{raw}")
}
