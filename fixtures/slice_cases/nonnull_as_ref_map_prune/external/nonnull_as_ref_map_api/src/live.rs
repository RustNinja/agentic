pub fn selected_nonnull_as_ref_map_report(raw: &str) -> String {
    nonnull_as_ref_map_model::selected_nonnull_as_ref_map(raw)
}

pub fn dead_live_nonnull_as_ref_map_report(raw: &str) -> String {
    format!("dead-live-nonnull-as-ref-map:{raw}")
}
