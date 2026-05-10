#[derive(Debug)]
pub struct ResultUnwrapErrUncheckedMapPayload {
    value: String,
}

impl ResultUnwrapErrUncheckedMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-unwrap-err-unchecked-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("result-unwrap-err-unchecked-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-result-unwrap-err-unchecked-map:{}", self.value)
    }
}

pub fn selected_result_unwrap_err_unchecked_map(raw: &str) -> String {
    let value: Result<(), ResultUnwrapErrUncheckedMapPayload> = Err(ResultUnwrapErrUncheckedMapPayload::new(raw));
    unsafe { value.unwrap_err_unchecked().render_label() }
}

pub fn dead_live_result_unwrap_err_unchecked_map(raw: &str) -> String {
    ResultUnwrapErrUncheckedMapPayload::new(raw).unused_label()
}
