use opensourced::opensourced;

#[opensourced]
pub fn selected_path_parent_map_report(raw: &str) -> String {
    path_parent_map_api::selected_path_parent_map_report(raw)
}

pub fn dead_path_parent_map_report(raw: &str) -> String {
    path_parent_map_api::dead_path_parent_map_report(raw)
}
