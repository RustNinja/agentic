pub fn selected_mpsc_recv_timeout_map_report(raw: &str) -> String {
    mpsc_recv_timeout_map_model::selected_mpsc_recv_timeout_map(raw)
}

pub fn dead_live_mpsc_recv_timeout_map_report(raw: &str) -> String {
    format!("dead-live-mpsc-recv-timeout-map-report:{raw}")
}
