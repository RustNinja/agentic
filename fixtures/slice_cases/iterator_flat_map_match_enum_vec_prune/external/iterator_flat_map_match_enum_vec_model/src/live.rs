pub struct IteratorFlatMapMatchEnumVecPayload {
    value: String,
}

impl IteratorFlatMapMatchEnumVecPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-flat-map-match-enum-vec:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-flat-map-match-enum-vec:{}", self.value)
    }
}

pub enum IteratorFlatMapMatchEnumVecCommand {
    Live(IteratorFlatMapMatchEnumVecPayload),
    Empty,
    Ignored,
}

pub fn selected_iterator_flat_map_match_enum_vec(raw: &str) -> String {
    let commands = vec![
        IteratorFlatMapMatchEnumVecCommand::Empty,
        IteratorFlatMapMatchEnumVecCommand::Live(IteratorFlatMapMatchEnumVecPayload::new(raw)),
        IteratorFlatMapMatchEnumVecCommand::Ignored,
    ];
    commands
        .into_iter()
        .flat_map(|command| match command {
            IteratorFlatMapMatchEnumVecCommand::Live(payload) => vec![payload],
            _ => Vec::new(),
        })
        .map(|payload| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_flat_map_match_enum_vec(raw: &str) -> String {
    IteratorFlatMapMatchEnumVecPayload::new(raw).unused_label()
}
