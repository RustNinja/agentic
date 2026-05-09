use opensourced::opensourced;

#[opensourced]
pub fn selected_option_as_deref_mut_map_report(raw: &str) -> String {
    option_as_deref_mut_api::selected_option_as_deref_mut_map_report(raw)
}

pub fn dead_option_as_deref_mut_map_report(raw: &str) -> String {
    option_as_deref_mut_api::dead_option_as_deref_mut_map_report(raw)
}
