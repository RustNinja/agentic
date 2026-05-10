use opensourced::opensourced;

#[opensourced]
pub fn selected_cstring_as_bytes_first_map_report(raw: &str) -> String {
    cstring_as_bytes_first_map_api::selected_cstring_as_bytes_first_map_report(raw)
}

pub fn dead_cstring_as_bytes_first_map_report(raw: &str) -> String {
    cstring_as_bytes_first_map_api::dead_cstring_as_bytes_first_map_report(raw)
}
