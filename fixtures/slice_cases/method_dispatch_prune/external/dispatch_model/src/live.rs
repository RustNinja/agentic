pub enum DispatchMethod {
    Start(StartParams),
    Unknown { method: String },
}

impl DispatchMethod {
    pub fn from_wire(method: &str, payload: &str) -> Self {
        match method {
            "start" => Self::Start(StartParams {
                label: payload.trim().to_string(),
            }),
            other => Self::Unknown {
                method: other.to_string(),
            },
        }
    }

    pub fn render(self) -> String {
        match self {
            Self::Start(params) => params.render(),
            Self::Unknown { method } => format!("unknown:{method}"),
        }
    }

    pub fn dead_method(self) -> String {
        "dead-dispatch".to_string()
    }
}

pub struct StartParams {
    label: String,
}

impl StartParams {
    pub fn render(self) -> String {
        format!("start:{}", self.label)
    }
}

pub fn selected_dispatch(method: &str, payload: &str) -> String {
    DispatchMethod::from_wire(method, payload).render()
}

pub fn dead_live_dispatch(raw: &str) -> String {
    format!("dead-live-dispatch:{raw}")
}
