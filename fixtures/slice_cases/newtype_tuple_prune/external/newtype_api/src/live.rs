pub fn selected_newtype_report(raw: &str) -> String {
    newtype_model::selected_newtype(raw).render()
}

pub fn dead_live_newtype_report(raw: &str) -> String {
    format!("dead-live-newtype:{raw}")
}
