pub fn selected_phantomdata_surface_map_report(raw: &str) -> String {
    phantomdata_surface_model::selected_phantomdata_surface_map(raw)
}

pub fn dead_live_phantomdata_surface_map_report(raw: &str) -> String {
    format!("dead-live-phantomdata-surface-map:{raw}")
}
