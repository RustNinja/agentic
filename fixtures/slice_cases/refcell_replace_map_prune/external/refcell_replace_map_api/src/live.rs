pub fn selected_refcell_replace_map_report(raw: &str) -> String {
    refcell_replace_map_model::selected_refcell_replace_map(raw)
}

pub fn dead_live_refcell_replace_map_report(raw: &str) -> String {
    format!("dead-live-refcell-replace-map:{raw}")
}
