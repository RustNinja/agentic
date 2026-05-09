use std::iter;

#[derive(Clone)]
pub struct IterFromFnMapPayload {
    value: String,
}

impl IterFromFnMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iter-from-fn-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("iter-from-fn-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iter-from-fn-map:{}", self.value)
    }
}

pub fn selected_iter_from_fn_map(raw: &str) -> String {
    let mut next = Some(IterFromFnMapPayload::new(raw));
    iter::from_fn(move || next.take())
        .next()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "iter-from-fn-map:missing".to_string())
}

pub fn dead_live_iter_from_fn_map(raw: &str) -> String {
    IterFromFnMapPayload::new(raw).dead_method()
}
