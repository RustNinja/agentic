use opensourced::opensourced;

#[opensourced]
pub fn selected_summary(seed: u32) -> String {
    support_records::selected_summary(seed)
}

pub fn dead_summary(seed: u32) -> String {
    support_records::dead_summary(seed)
}

