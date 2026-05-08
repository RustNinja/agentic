use opensourced::opensourced;

#[opensourced]
pub fn selected_chain_report(raw: &str) -> String {
    chain_api::selected_chain_report(raw)
}

pub fn dead_chain_report(raw: &str) -> String {
    chain_api::dead_chain_report(raw)
}
