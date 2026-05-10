#[derive(Debug)]
pub struct ResultExpectErrMapPayload {
    value: String,
}

impl ResultExpectErrMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-expect-err-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("result-expect-err-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-result-expect-err-map:{}", self.value)
    }
}

pub fn selected_result_expect_err_map(raw: &str) -> String {
    let value: Result<(), ResultExpectErrMapPayload> = Err(ResultExpectErrMapPayload::new(raw));
    value.expect_err("expected payload").render_label()
}

pub fn dead_live_result_expect_err_map(raw: &str) -> String {
    ResultExpectErrMapPayload::new(raw).unused_label()
}
