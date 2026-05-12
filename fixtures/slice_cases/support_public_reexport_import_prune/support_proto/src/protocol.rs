pub use crate::approvals::DeadAmendment;
pub use crate::approvals::ExecPolicyAmendment;
pub use crate::approvals::NetworkPolicyAmendment;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReviewDecision {
    ApprovedExecpolicyAmendment {
        proposed_execpolicy_amendment: ExecPolicyAmendment,
    },
    NetworkPolicyAmendment {
        network_policy_amendment: NetworkPolicyAmendment,
    },
    Approved,
}

pub fn review_label(decision: ReviewDecision) -> String {
    match decision {
        ReviewDecision::ApprovedExecpolicyAmendment {
            proposed_execpolicy_amendment,
        } => proposed_execpolicy_amendment.command().join(" "),
        ReviewDecision::NetworkPolicyAmendment {
            network_policy_amendment,
        } => network_policy_amendment.host,
        ReviewDecision::Approved => "approved".to_string(),
    }
}

pub fn dead_label() -> String {
    let dead = DeadAmendment {
        note: "dead".to_string(),
    };
    dead.note
}

