#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecPolicyAmendment {
    pub command: Vec<String>,
}

impl ExecPolicyAmendment {
    pub fn command(&self) -> &[String] {
        &self.command
    }
}

impl From<Vec<String>> for ExecPolicyAmendment {
    fn from(command: Vec<String>) -> Self {
        Self { command }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkPolicyAmendment {
    pub host: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeadAmendment {
    pub note: String,
}

