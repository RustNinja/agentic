use opensourced::opensourced;

#[opensourced]
pub fn selected_cow_borrowed_as_ref_map_report(raw: &str) -> String {
    cow_borrowed_as_ref_api::selected_cow_borrowed_as_ref_map_report(raw)
}

pub fn dead_cow_borrowed_as_ref_map_report(raw: &str) -> String {
    cow_borrowed_as_ref_api::dead_cow_borrowed_as_ref_map_report(raw)
}
