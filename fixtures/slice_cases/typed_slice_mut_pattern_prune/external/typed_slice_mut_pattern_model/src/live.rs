pub struct TypedSliceMutPatternPayload {
    value: String,
}

impl TypedSliceMutPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("typed-slice-mut-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("typed-slice-mut-pattern:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-typed-slice-mut-pattern:{}", self.value)
    }
}

fn typed_slice_mut_pattern_items(raw: &str) -> Vec<TypedSliceMutPatternPayload> {
    vec![TypedSliceMutPatternPayload::new(raw), TypedSliceMutPatternPayload::new("tail")]
}

pub fn selected_typed_slice_mut_pattern(raw: &str) -> String {
    let mut items = typed_slice_mut_pattern_items(raw);
    let [payload, ..]: &mut [TypedSliceMutPatternPayload] = items.as_mut_slice() else {
        return "typed-slice-mut-pattern:missing".to_string();
    };
    payload.bump_and_render()
}

pub fn dead_live_typed_slice_mut_pattern(raw: &str) -> String {
    TypedSliceMutPatternPayload::new(raw).unused_label()
}
