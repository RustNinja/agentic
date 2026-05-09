pub fn selected_iterator_unzip_pair_map_report(raw: &str) -> String {
    iterator_unzip_pair_map_model::selected_iterator_unzip_pair_map(raw)
}

pub fn dead_live_iterator_unzip_pair_map_report(raw: &str) -> String {
    format!("dead-live-iterator-unzip-pair-map:{raw}")
}
