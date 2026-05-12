#[opensourced::opensourced]
pub fn selected_reason() -> &'static str {
    support_proto::experimental_api::registered_reason()
}
