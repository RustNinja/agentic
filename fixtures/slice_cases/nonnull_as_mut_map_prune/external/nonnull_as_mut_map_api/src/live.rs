pub fn selected_nonnull_as_mut_map_report(raw: &str) -> String {
    nonnull_as_mut_map_model::selected_nonnull_as_mut_map(raw)
}

pub fn dead_live_nonnull_as_mut_map_report(raw: &str) -> String {
    format!("dead-nonnull-as-mut-map-live-report:{raw}")
}
