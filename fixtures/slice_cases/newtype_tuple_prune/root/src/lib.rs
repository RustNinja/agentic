use opensourced::opensourced;

#[opensourced]
pub fn selected_newtype_report(raw: &str) -> String {
    newtype_api::selected_newtype_report(raw)
}

pub fn dead_newtype_report(raw: &str) -> String {
    newtype_api::dead_newtype_report(raw)
}
