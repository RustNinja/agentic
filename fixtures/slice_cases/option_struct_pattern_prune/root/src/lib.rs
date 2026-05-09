use opensourced::opensourced;

#[opensourced]
pub fn selected_option_struct_report(raw: &str) -> String {
    option_struct_api::selected_option_struct_report(raw)
}

pub fn dead_option_struct_report(raw: &str) -> String {
    option_struct_api::dead_option_struct_report(raw)
}
