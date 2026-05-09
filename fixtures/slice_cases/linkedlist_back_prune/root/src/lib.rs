use opensourced::opensourced;

#[opensourced]
pub fn selected_linkedlist_back_report(raw: &str) -> String {
    linkedlist_back_api::selected_linkedlist_back_report(raw)
}

pub fn dead_linkedlist_back_report(raw: &str) -> String {
    linkedlist_back_api::dead_linkedlist_back_report(raw)
}
