use opensourced::opensourced;

#[opensourced]
pub fn selected_struct_inspect_report(raw: &str) -> String {
    struct_inspect_api::selected_struct_inspect_report(raw)
}

pub fn dead_struct_inspect_report(raw: &str) -> String {
    struct_inspect_api::dead_struct_inspect_report(raw)
}
