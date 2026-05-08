use opensourced::opensourced;

#[opensourced]
pub fn selected_callback_score(value: u32) -> u32 {
    callback_api::selected_callback_score(value)
}

pub fn dead_callback_score(value: u32) -> u32 {
    callback_api::dead_callback_score(value)
}
