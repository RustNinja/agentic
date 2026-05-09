pub fn selected_cow_to_mut_map_report(raw: &str) -> String {
    cow_to_mut_model::selected_cow_to_mut_map(raw)
}

pub fn dead_live_cow_to_mut_map_report(raw: &str) -> String {
    format!("dead-live-cow-to-mut-map:{raw}")
}
