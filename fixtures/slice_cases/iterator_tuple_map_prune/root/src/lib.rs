use opensourced::opensourced;

#[opensourced]
pub fn selected_tuple_map_report(raw: &str) -> String {
    tuple_map_api::selected_tuple_map_report(raw)
}

pub fn dead_tuple_map_report(raw: &str) -> String {
    tuple_map_api::dead_tuple_map_report(raw)
}
