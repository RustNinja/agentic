use opensourced::opensourced;

#[opensourced]
pub fn selected_mpsc_iter_next_map_report(raw: &str) -> String {
    mpsc_iter_next_map_api::selected_mpsc_iter_next_map_report(raw)
}

pub fn dead_mpsc_iter_next_map_report(raw: &str) -> String {
    mpsc_iter_next_map_api::dead_mpsc_iter_next_map_report(raw)
}
