use opensourced::opensourced;

#[opensourced]
pub fn selected_linkedlist_iter_find_report(raw: &str) -> String {
    linkedlist_iter_find_api::selected_linkedlist_iter_find_report(raw)
}

pub fn dead_linkedlist_iter_find_report(raw: &str) -> String {
    linkedlist_iter_find_api::dead_linkedlist_iter_find_report(raw)
}
