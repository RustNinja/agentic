pub fn selected_hashset_intersection_report(raw: &str) -> String {
    hashset_intersection_model::selected_hashset_intersection(raw)
}

pub fn dead_live_hashset_intersection_report(raw: &str) -> String {
    format!("dead-live-hashset-intersection:{raw}")
}
