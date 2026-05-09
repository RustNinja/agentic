use opensourced::opensourced;

#[opensourced]
pub fn selected_cow_to_mut_map_report(raw: &str) -> String {
    cow_to_mut_api::selected_cow_to_mut_map_report(raw)
}

pub fn dead_cow_to_mut_map_report(raw: &str) -> String {
    cow_to_mut_api::dead_cow_to_mut_map_report(raw)
}
