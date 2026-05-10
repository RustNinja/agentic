use opensourced::opensourced;

#[opensourced]
pub fn selected_typed_array_mut_pattern_report(raw: &str) -> String {
    typed_array_mut_pattern_api::selected_typed_array_mut_pattern_report(raw)
}

pub fn dead_typed_array_mut_pattern_report(raw: &str) -> String {
    typed_array_mut_pattern_api::dead_typed_array_mut_pattern_report(raw)
}
