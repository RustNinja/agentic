pub fn selected_borrow_trait_map_report(raw: &str) -> String {
    borrow_trait_model::selected_borrow_trait_map(raw)
}

pub fn dead_live_borrow_trait_map_report(raw: &str) -> String {
    format!("dead-live-borrow-trait-map:{raw}")
}
