use opensourced::opensourced;

#[opensourced]
pub fn selected_iter_successors_map_report(raw: &str) -> String {
    iter_successors_map_api::selected_iter_successors_map_report(raw)
}

pub fn dead_iter_successors_map_report(raw: &str) -> String {
    iter_successors_map_api::dead_iter_successors_map_report(raw)
}
