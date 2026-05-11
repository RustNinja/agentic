use opensourced::opensourced;

#[opensourced]
pub fn selected_summary(seed: u32) -> String {
    field_api::selected(seed)
}

pub fn dead_summary(seed: u32) -> String {
    field_api::dead(seed)
}

