pub fn selected_mpsc_recv_unwrap_map_report(raw: &str) -> String {
    mpsc_recv_unwrap_map_model::selected_mpsc_recv_unwrap_map(raw)
}

pub fn dead_live_mpsc_recv_unwrap_map_report(raw: &str) -> String {
    format!("dead-live-mpsc-recv-unwrap-map-report:{raw}")
}
