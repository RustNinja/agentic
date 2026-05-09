pub fn selected_vec_dedup_by_key_report(raw: &str) -> String {
    vec_dedup_by_key_model::selected_vec_dedup_by_key(raw)
}

pub fn dead_live_vec_dedup_by_key_report(raw: &str) -> String {
    format!("dead-vec-dedup-by-key-live-report:{raw}")
}
