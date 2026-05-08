use opensourced::opensourced;

#[opensourced]
pub fn selected_trait_report(raw: &str) -> String {
    trait_api::selected_trait_report(raw)
}

pub fn dead_trait_report(raw: &str) -> String {
    trait_api::dead_trait_report(raw)
}
