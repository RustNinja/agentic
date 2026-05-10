pub struct TypedSliceEnumStructPatternPayload {
    value: String,
}

impl TypedSliceEnumStructPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("typed-slice-enum-struct-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("typed-slice-enum-struct-pattern:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-typed-slice-enum-struct-pattern:{}", self.value)
    }
}

pub enum TypedSliceEnumStructPatternEnvelope {
    Live { payload: TypedSliceEnumStructPatternPayload },
    Dead,
}

pub fn selected_typed_slice_enum_struct_pattern(raw: &str) -> String {
    let [TypedSliceEnumStructPatternEnvelope::Live { payload }, ..]: [TypedSliceEnumStructPatternEnvelope; 1] = [TypedSliceEnumStructPatternEnvelope::Live {
        payload: TypedSliceEnumStructPatternPayload::new(raw),
    }] else {
        return "typed-slice-enum-struct-pattern:missing".to_string();
    };
    payload.render_label()
}

pub fn dead_live_typed_slice_enum_struct_pattern(raw: &str) -> String {
    TypedSliceEnumStructPatternPayload::new(raw).unused_label()
}
