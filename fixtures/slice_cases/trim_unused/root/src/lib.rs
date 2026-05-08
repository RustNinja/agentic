use opensourced::opensourced;

#[opensourced]
pub fn selected_total(value: &str) -> String {
    used::format_live(value)
}

pub fn dead_entry(value: &str) -> String {
    unused::format_dead(value)
}

