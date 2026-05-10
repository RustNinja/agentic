pub fn selected_sync_mpsc_recv_map_report(raw: &str) -> String {
    sync_mpsc_recv_map_model::selected_sync_mpsc_recv_map(raw)
}

pub fn dead_live_sync_mpsc_recv_map_report(raw: &str) -> String {
    format!("dead-live-sync-mpsc-recv-map-report:{raw}")
}
