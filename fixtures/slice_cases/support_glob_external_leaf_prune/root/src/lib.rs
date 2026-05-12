use opensourced::opensourced;

#[opensourced]
pub fn selected_leaf_report(raw: &str) -> String {
    support_package::selected_leaf_report(raw)
}

pub fn dead_leaf_report(raw: &str) -> String {
    support_package::dead_leaf_report(raw)
}
