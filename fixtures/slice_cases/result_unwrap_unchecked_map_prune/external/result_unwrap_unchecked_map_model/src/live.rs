pub struct ResultUnwrapUncheckedMapPayload {
    value: String,
}

impl ResultUnwrapUncheckedMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-unwrap-unchecked-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("result-unwrap-unchecked-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-result-unwrap-unchecked-map:{}", self.value)
    }
}

pub fn selected_result_unwrap_unchecked_map(raw: &str) -> String {
    let value: Result<ResultUnwrapUncheckedMapPayload, ()> = Ok(ResultUnwrapUncheckedMapPayload::new(raw));
    unsafe { value.unwrap_unchecked().render_label() }
}

pub fn dead_live_result_unwrap_unchecked_map(raw: &str) -> String {
    ResultUnwrapUncheckedMapPayload::new(raw).unused_label()
}
