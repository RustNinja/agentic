#[derive(Debug)]
pub struct ResultUnwrapErrMapPayload {
    value: String,
}

impl ResultUnwrapErrMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-unwrap-err-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("result-unwrap-err-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-result-unwrap-err-map:{}", self.value)
    }
}

pub fn selected_result_unwrap_err_map(raw: &str) -> String {
    let value: Result<(), ResultUnwrapErrMapPayload> = Err(ResultUnwrapErrMapPayload::new(raw));
    value.unwrap_err().render_label()
}

pub fn dead_live_result_unwrap_err_map(raw: &str) -> String {
    ResultUnwrapErrMapPayload::new(raw).unused_label()
}
