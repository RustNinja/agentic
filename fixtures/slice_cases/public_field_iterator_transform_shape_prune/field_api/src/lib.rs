pub fn selected_summary(seed: u32) -> String {
    let snapshot = source_record::snapshot(seed);

    let titles = snapshot
        .records
        .iter()
        .map(|record| view_record::ViewRecord {
            title: record.raw_label.clone(),
            score: record.raw_value,
            loop_title: record.raw_label.to_uppercase(),
            dead_view_note: None,
        })
        .filter(|view| view.score > 0)
        .map(|view| view.title)
        .collect::<Vec<_>>()
        .join(",");

    let filtered = snapshot
        .records
        .iter()
        .filter_map(|record| {
            Some(view_record::FilterRecord {
                code: record.filter_code.clone(),
                weight: record.raw_value,
                dead_filter_note: None,
            })
        })
        .map(|view| format!("{}:{}", view.code, view.weight))
        .collect::<Vec<_>>()
        .join(",");

    let mut loop_titles = Vec::new();
    for view in snapshot.records.iter().map(|record| view_record::ViewRecord {
        title: record.raw_label.clone(),
        score: record.raw_value,
        loop_title: record.raw_label.to_lowercase(),
        dead_view_note: None,
    }) {
        loop_titles.push(view.loop_title);
    }

    let children = snapshot
        .records
        .iter()
        .flat_map(|record| record.children.iter())
        .map(|child| view_record::ChildView {
            child_label: child.child_label.clone(),
            child_weight: child.child_weight,
            dead_child_view_note: None,
        })
        .filter(|child| child.child_weight > 0)
        .map(|child| child.child_label)
        .collect::<Vec<_>>()
        .join(",");

    let local_views = snapshot
        .records
        .iter()
        .map(|record| view_record::LocalView {
            local_title: record.raw_label.clone(),
            local_score: record.raw_value,
            dead_local_note: None,
        })
        .collect::<Vec<_>>();
    let local = local_views
        .iter()
        .map(|view| format!("{}:{}", view.local_title, view.local_score))
        .collect::<Vec<_>>()
        .join(",");

    let lazy_views = snapshot.records.iter().map(|record| view_record::LazyView {
        lazy_title: record.raw_label.clone(),
        lazy_score: record.raw_value,
        dead_lazy_note: None,
    });
    let lazy = lazy_views
        .map(|view| format!("{}:{}", view.lazy_title, view.lazy_score))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{titles}:{filtered}:{}:{children}:{local}:{lazy}",
        loop_titles.join(",")
    )
}

pub fn dead_summary(seed: u32) -> String {
    let snapshot = source_record::dead_snapshot(seed);
    snapshot
        .unused_note
        .into_iter()
        .chain(snapshot.records.into_iter().filter_map(|record| record.dead_note))
        .collect::<Vec<_>>()
        .join(",")
}
