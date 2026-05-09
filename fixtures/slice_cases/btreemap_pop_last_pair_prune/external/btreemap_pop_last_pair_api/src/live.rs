pub fn selected_btreemap_pop_last_pair_report(raw: &str) -> String {
    btreemap_pop_last_pair_model::selected_btreemap_pop_last_pair(raw)
}

pub fn dead_live_btreemap_pop_last_pair_report(raw: &str) -> String {
    format!("dead-live-btreemap-pop-last-pair:{raw}")
}
