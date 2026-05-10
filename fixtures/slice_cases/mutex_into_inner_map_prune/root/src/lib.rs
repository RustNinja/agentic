use opensourced::opensourced;

#[opensourced]
pub fn selected_mutex_into_inner_map_report(raw: &str) -> String {
    mutex_into_inner_map_api::selected_mutex_into_inner_map_report(raw)
}

pub fn dead_mutex_into_inner_map_report(raw: &str) -> String {
    mutex_into_inner_map_api::dead_mutex_into_inner_map_report(raw)
}
