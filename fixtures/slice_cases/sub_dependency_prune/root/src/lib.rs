use opensourced::opensourced;

#[opensourced]
pub fn selected_summary(value: &str) -> String {
    adapter::selected_bridge(value)
}

pub fn dead_summary(value: &str) -> String {
    adapter::dead_bridge(value)
}

