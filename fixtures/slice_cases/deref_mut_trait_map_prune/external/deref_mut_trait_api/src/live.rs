pub fn selected_deref_mut_trait_map_report(raw: &str) -> String {
    deref_mut_trait_model::selected_deref_mut_trait_map(raw)
}

pub fn dead_live_deref_mut_trait_map_report(raw: &str) -> String {
    format!("dead-live-deref-mut-trait-map:{raw}")
}
