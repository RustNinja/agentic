use opensourced::opensourced;

#[opensourced]
pub fn selected_nested_tuple_map_report(raw: &str) -> String {
    nested_tuple_api::selected_nested_tuple_map_report(raw)
}

pub fn dead_nested_tuple_map_report(raw: &str) -> String {
    nested_tuple_api::dead_nested_tuple_map_report(raw)
}
