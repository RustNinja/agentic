use opensourced::opensourced;

#[opensourced]
pub fn selected_adjacent_summary(raw: &str) -> String {
    serde_adjacent_api::selected_adjacent_summary(raw)
}

pub fn dead_adjacent_summary(raw: &str) -> String {
    serde_adjacent_api::dead_adjacent_summary(raw)
}
