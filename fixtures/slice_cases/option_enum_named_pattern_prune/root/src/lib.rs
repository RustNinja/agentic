use opensourced::opensourced;

#[opensourced]
pub fn selected_option_enum_named_report(raw: &str) -> String {
    option_enum_named_api::selected_option_enum_named_report(raw)
}

pub fn dead_option_enum_named_report(raw: &str) -> String {
    option_enum_named_api::dead_option_enum_named_report(raw)
}
