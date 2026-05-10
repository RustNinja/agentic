pub fn selected_mpsc_try_recv_map_report(raw: &str) -> String {
    mpsc_try_recv_map_model::selected_mpsc_try_recv_map(raw)
}

pub fn dead_live_mpsc_try_recv_map_report(raw: &str) -> String {
    format!("dead-live-mpsc-try-recv-map-report:{raw}")
}
