pub struct TypedSliceNamedStructPatternPayload {
    value: String,
}

impl TypedSliceNamedStructPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("typed-slice-named-struct-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("typed-slice-named-struct-pattern:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-typed-slice-named-struct-pattern:{}", self.value)
    }
}

pub struct TypedSliceNamedStructPatternEnvelope {
    payload: TypedSliceNamedStructPatternPayload,
    ignored: usize,
}

pub fn selected_typed_slice_named_struct_pattern(raw: &str) -> String {
    let [TypedSliceNamedStructPatternEnvelope { payload, ignored: _ }, ..]: [TypedSliceNamedStructPatternEnvelope; 1] = [TypedSliceNamedStructPatternEnvelope {
        payload: TypedSliceNamedStructPatternPayload::new(raw),
        ignored: 1,
    }];
    payload.render_label()
}

pub fn dead_live_typed_slice_named_struct_pattern(raw: &str) -> String {
    TypedSliceNamedStructPatternPayload::new(raw).unused_label()
}
