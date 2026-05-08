use std::convert::TryFrom;

pub struct WireSpec {
    raw: String,
}

pub struct DynamicSpec {
    label: String,
}

impl DynamicSpec {
    pub fn render(self) -> String {
        format!("dynamic:{}", self.label)
    }

    pub fn dead_method(self) -> String {
        format!("dead-transpose:{}", self.label)
    }
}

pub struct TransposeError {
    message: String,
}

impl TransposeError {
    pub fn render(self) -> String {
        format!("transpose-error:{}", self.message)
    }

    pub fn dead_method(self) -> String {
        format!("dead-error:{}", self.message)
    }
}

impl TryFrom<WireSpec> for DynamicSpec {
    type Error = TransposeError;

    fn try_from(value: WireSpec) -> Result<Self, Self::Error> {
        if value.raw.trim().is_empty() {
            Err(TransposeError {
                message: "empty".to_string(),
            })
        } else {
            Ok(Self {
                label: value.raw.trim().to_string(),
            })
        }
    }
}

pub fn parse_optional_specs(raw: &str) -> Result<Option<Vec<DynamicSpec>>, TransposeError> {
    let wires = if raw.trim() == "none" {
        None
    } else {
        Some(
            raw.split(',')
                .map(|part| WireSpec {
                    raw: part.to_string(),
                })
                .collect::<Vec<_>>(),
        )
    };

    wires
        .map(|values| {
            values
                .into_iter()
                .map(DynamicSpec::try_from)
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()
}

pub fn dead_live_transpose(raw: &str) -> String {
    DynamicSpec {
        label: raw.to_string(),
    }
    .dead_method()
}
