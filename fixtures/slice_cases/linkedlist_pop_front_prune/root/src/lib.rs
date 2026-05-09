use opensourced::opensourced;

#[opensourced]
pub fn selected_linkedlist_pop_front_report(raw: &str) -> String {
    linkedlist_pop_front_api::selected_linkedlist_pop_front_report(raw)
}

pub fn dead_linkedlist_pop_front_report(raw: &str) -> String {
    linkedlist_pop_front_api::dead_linkedlist_pop_front_report(raw)
}
