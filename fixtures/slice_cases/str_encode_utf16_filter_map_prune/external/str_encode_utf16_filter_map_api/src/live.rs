pub fn selected_str_encode_utf16_filter_map_report(raw: &str) -> String {
    str_encode_utf16_filter_map_model::selected_str_encode_utf16_filter_map(raw)
}

pub fn dead_live_str_encode_utf16_filter_map_report(raw: &str) -> String {
    format!("dead-str-encode-utf16-filter-map-live-report:{raw}")
}
