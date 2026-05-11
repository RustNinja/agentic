use opensourced::opensourced;

#[opensourced]
pub fn selected_sum(seed: u32) -> u32 {
    field_api::selected_sum(seed)
}

pub fn dead_sum(seed: u32) -> String {
    field_api::dead_sum(seed)
}

