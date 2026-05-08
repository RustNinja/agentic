pub enum PatchOp {
    Add,
    Remove,
}

pub enum PatchError {
    Empty,
}

pub struct PatchSegment {
    value: String,
}

impl TryFrom<&str> for PatchSegment {
    type Error = PatchError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        if value.is_empty() {
            Err(PatchError::Empty)
        } else {
            Ok(Self {
                value: value.to_string(),
            })
        }
    }
}

impl PatchSegment {
    pub fn render(&self) -> String {
        self.value.clone()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-segment:{}", self.value)
    }
}

pub struct PatchRequest {
    pub op: PatchOp,
    pub path: Vec<PatchSegment>,
    pub value: Option<String>,
}

pub fn build_patch(raw: &str) -> PatchRequest {
    let segment = PatchSegment::try_from(raw).unwrap_or(PatchSegment {
        value: "fallback".to_string(),
    });
    PatchRequest {
        op: PatchOp::Add,
        path: vec![segment],
        value: Some(raw.trim().to_string()),
    }
}

pub fn selected_patch(raw: &str) -> String {
    let patch = build_patch(raw);
    let head = patch.path.first().map(PatchSegment::render).unwrap_or_default();
    match patch.op {
        PatchOp::Add => format!("add:{head}:{}", patch.value.unwrap_or_default()),
        PatchOp::Remove => format!("remove:{head}"),
    }
}

pub fn dead_live_patch(raw: &str) -> String {
    PatchSegment::try_from(raw)
        .map(|segment| segment.dead_method())
        .unwrap_or_else(|_| "dead-empty".to_string())
}
