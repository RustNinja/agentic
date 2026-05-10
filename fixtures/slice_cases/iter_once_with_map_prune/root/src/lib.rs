use opensourced::opensourced;

#[opensourced]
pub fn selected_iter_once_with_map_report(raw: &str) -> String {
    iter_once_with_map_api::selected_iter_once_with_map_report(raw)
}

pub fn dead_iter_once_with_map_report(raw: &str) -> String {
    iter_once_with_map_api::dead_iter_once_with_map_report(raw)
}
