use opensourced::opensourced;

#[opensourced]
pub fn selected_cow_owned_into_owned_map_report(raw: &str) -> String {
    cow_owned_into_owned_api::selected_cow_owned_into_owned_map_report(raw)
}

pub fn dead_cow_owned_into_owned_map_report(raw: &str) -> String {
    cow_owned_into_owned_api::dead_cow_owned_into_owned_map_report(raw)
}
