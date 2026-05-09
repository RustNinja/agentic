pub fn selected_as_mut_trait_map_report(raw: &str) -> String {
    as_mut_trait_model::selected_as_mut_trait_map(raw)
}

pub fn dead_live_as_mut_trait_map_report(raw: &str) -> String {
    format!("dead-live-as-mut-trait-map:{raw}")
}
