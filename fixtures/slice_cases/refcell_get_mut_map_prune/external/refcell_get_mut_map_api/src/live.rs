pub fn selected_refcell_get_mut_map_report(raw: &str) -> String {
    refcell_get_mut_map_model::selected_refcell_get_mut_map(raw)
}

pub fn dead_live_refcell_get_mut_map_report(raw: &str) -> String {
    format!("dead-refcell-get-mut-map-live-report:{raw}")
}
