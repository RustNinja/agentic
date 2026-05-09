pub fn selected_cow_borrowed_as_ref_map_report(raw: &str) -> String {
    cow_borrowed_as_ref_model::selected_cow_borrowed_as_ref_map(raw)
}

pub fn dead_live_cow_borrowed_as_ref_map_report(raw: &str) -> String {
    format!("dead-live-cow-borrowed-as-ref-map:{raw}")
}
