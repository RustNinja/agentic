use opensourced::opensourced;

#[opensourced]
pub fn selected_generated_total() -> u32 {
    generated_support::selected_generated_value()
}

pub fn dead_generated_total() -> u32 {
    generated_support::dead_generated_value()
}

