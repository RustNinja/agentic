use opensourced::opensourced;

#[opensourced]
pub fn selected_wire(raw: &str) -> String {
    api::selected_wire(raw)
}

pub fn dead_wire(raw: &str) -> String {
    api::dead_wire(raw)
}
