use opensourced::opensourced;

#[opensourced]
pub fn selected_score(seed: u32) -> u32 {
    field_api::selected_score(seed)
}

pub fn dead_score(seed: u32) -> String {
    field_api::dead_score(seed)
}

