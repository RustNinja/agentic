use opensourced::opensourced;

#[opensourced]
pub fn selected_linkedlist_into_iter_report(raw: &str) -> String {
    linkedlist_into_iter_api::selected_linkedlist_into_iter_report(raw)
}

pub fn dead_linkedlist_into_iter_report(raw: &str) -> String {
    linkedlist_into_iter_api::dead_linkedlist_into_iter_report(raw)
}
