pub fn selected_maybeuninit_assume_init_map_report(raw: &str) -> String {
    maybeuninit_assume_init_map_model::selected_maybeuninit_assume_init_map(raw)
}

pub fn dead_live_maybeuninit_assume_init_map_report(raw: &str) -> String {
    format!("dead-live-maybeuninit-assume-init-map:{raw}")
}
