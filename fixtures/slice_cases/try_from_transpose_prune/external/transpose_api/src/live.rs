pub fn selected_transpose_report(raw: &str) -> String {
    transpose_model::parse_optional_specs(raw)
        .map(|specs| {
            specs
                .map(|items| {
                    items
                        .into_iter()
                        .map(|item| item.render())
                        .collect::<Vec<_>>()
                        .join("|")
                })
                .unwrap_or_else(|| "none".to_string())
        })
        .unwrap_or_else(transpose_model::TransposeError::render)
}

pub fn dead_live_transpose_report(raw: &str) -> String {
    format!("dead-live-transpose:{raw}")
}
