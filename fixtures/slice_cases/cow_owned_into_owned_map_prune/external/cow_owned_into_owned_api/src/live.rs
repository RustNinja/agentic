pub fn selected_cow_owned_into_owned_map_report(raw: &str) -> String {
    cow_owned_into_owned_model::selected_cow_owned_into_owned_map(raw)
}

pub fn dead_live_cow_owned_into_owned_map_report(raw: &str) -> String {
    format!("dead-live-cow-owned-into-owned-map:{raw}")
}
