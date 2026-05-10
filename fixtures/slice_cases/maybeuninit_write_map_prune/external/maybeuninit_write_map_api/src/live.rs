pub fn selected_maybeuninit_write_map_report(raw: &str) -> String {
    maybeuninit_write_map_model::selected_maybeuninit_write_map(raw)
}

pub fn dead_live_maybeuninit_write_map_report(raw: &str) -> String {
    format!("dead-maybeuninit-write-map-live-report:{raw}")
}
