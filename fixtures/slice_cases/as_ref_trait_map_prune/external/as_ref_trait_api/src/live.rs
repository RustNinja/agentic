pub fn selected_as_ref_trait_map_report(raw: &str) -> String {
    as_ref_trait_model::selected_as_ref_trait_map(raw)
}

pub fn dead_live_as_ref_trait_map_report(raw: &str) -> String {
    format!("dead-live-as-ref-trait-map:{raw}")
}
