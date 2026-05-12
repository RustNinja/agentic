#[opensourced::opensourced]
pub fn selected_review() -> String {
    support_proto::protocol::review_label(
        support_proto::protocol::ReviewDecision::ApprovedExecpolicyAmendment {
            proposed_execpolicy_amendment: vec!["cargo".to_string(), "check".to_string()].into(),
        },
    )
}

pub fn dead_review() -> String {
    support_proto::protocol::dead_label()
}
