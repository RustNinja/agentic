pub fn selected_hashset_union_report(raw: &str) -> String {
    hashset_union_model::selected_hashset_union(raw)
}

pub fn dead_live_hashset_union_report(raw: &str) -> String {
    format!("dead-live-hashset-union:{raw}")
}
