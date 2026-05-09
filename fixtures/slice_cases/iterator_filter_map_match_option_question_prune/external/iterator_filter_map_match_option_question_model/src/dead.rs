pub struct DeadIteratorFilterMapMatchOptionQuestionItem {
    value: String,
}

impl DeadIteratorFilterMapMatchOptionQuestionItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!(
            "dead-iterator-filter-map-match-option-question:{}",
            self.value
        )
    }
}

pub fn dead_iterator_filter_map_match_option_question(raw: &str) -> String {
    DeadIteratorFilterMapMatchOptionQuestionItem::new(raw).dead_method()
}
