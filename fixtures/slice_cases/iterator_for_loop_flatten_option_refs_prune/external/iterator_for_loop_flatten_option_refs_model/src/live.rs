pub struct IteratorForLoopFlattenOptionRefsPayload {
    value: String,
}

impl IteratorForLoopFlattenOptionRefsPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-for-loop-flatten-option-refs:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":bumped");
        self.render_label()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-for-loop-flatten-option-refs:{}", self.value)
    }
}

pub fn selected_iterator_for_loop_flatten_option_refs(raw: &str) -> String {
    let left = Some(IteratorForLoopFlattenOptionRefsPayload::new(raw));
    let right = None;
    let mut rendered = Vec::new();
    for payload in [&left, &right].into_iter().flatten() {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

pub fn dead_live_iterator_for_loop_flatten_option_refs(raw: &str) -> String {
    IteratorForLoopFlattenOptionRefsPayload::new(raw).dead_method()
}
