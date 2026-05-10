pub struct TypedSliceRefPatternPayload {
    value: String,
}

impl TypedSliceRefPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("typed-slice-ref-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("typed-slice-ref-pattern:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-typed-slice-ref-pattern:{}", self.value)
    }
}

fn typed_slice_ref_pattern_items(raw: &str) -> Vec<TypedSliceRefPatternPayload> {
    vec![TypedSliceRefPatternPayload::new(raw), TypedSliceRefPatternPayload::new("tail")]
}

pub fn selected_typed_slice_ref_pattern(raw: &str) -> String {
    let items = typed_slice_ref_pattern_items(raw);
    let [payload, ..]: &[TypedSliceRefPatternPayload] = items.as_slice() else {
        return "typed-slice-ref-pattern:missing".to_string();
    };
    payload.render_label()
}

pub fn dead_live_typed_slice_ref_pattern(raw: &str) -> String {
    TypedSliceRefPatternPayload::new(raw).unused_label()
}
