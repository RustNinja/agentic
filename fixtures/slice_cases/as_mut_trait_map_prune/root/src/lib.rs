use opensourced::opensourced;

#[opensourced]
pub fn selected_as_mut_trait_map_report(raw: &str) -> String {
    as_mut_trait_api::selected_as_mut_trait_map_report(raw)
}

pub fn dead_as_mut_trait_map_report(raw: &str) -> String {
    as_mut_trait_api::dead_as_mut_trait_map_report(raw)
}
