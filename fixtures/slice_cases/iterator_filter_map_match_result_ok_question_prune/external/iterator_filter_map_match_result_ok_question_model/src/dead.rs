pub struct DeadIteratorFilterMapMatchResultOkQuestionItem {
    value: String,
}

impl DeadIteratorFilterMapMatchResultOkQuestionItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!(
            "dead-iterator-filter-map-match-result-ok-question:{}",
            self.value
        )
    }
}

pub fn dead_iterator_filter_map_match_result_ok_question(raw: &str) -> String {
    DeadIteratorFilterMapMatchResultOkQuestionItem::new(raw).dead_method()
}
