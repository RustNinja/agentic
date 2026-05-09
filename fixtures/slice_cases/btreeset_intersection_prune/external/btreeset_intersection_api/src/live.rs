pub fn selected_btreeset_intersection_report(raw: &str) -> String {
    btreeset_intersection_model::selected_btreeset_intersection(raw)
}

pub fn dead_live_btreeset_intersection_report(raw: &str) -> String {
    format!("dead-live-btreeset-intersection:{raw}")
}
