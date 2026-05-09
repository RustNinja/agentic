use opensourced::opensourced;

#[opensourced]
pub fn selected_deref_mut_trait_map_report(raw: &str) -> String {
    deref_mut_trait_api::selected_deref_mut_trait_map_report(raw)
}

pub fn dead_deref_mut_trait_map_report(raw: &str) -> String {
    deref_mut_trait_api::dead_deref_mut_trait_map_report(raw)
}
