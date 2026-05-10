pub fn selected_hashset_shrink_to_fit_iter_map_report(raw: &str) -> String {
    hashset_shrink_to_fit_iter_map_model::selected_hashset_shrink_to_fit_iter_map(raw)
}

pub fn dead_live_hashset_shrink_to_fit_iter_map_report(raw: &str) -> String {
    format!("dead-hashset-shrink-to-fit-iter-map-live-report:{raw}")
}
