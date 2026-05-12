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

    let returned_views = build_returned_views(&snapshot);
    let returned = returned_views
        .iter()
        .map(|view| format!("{}:{}", view.returned_title, view.returned_score))
        .collect::<Vec<_>>()
        .join(",");

    let factory = ViewFactory {
        prefix: "method".to_string(),
        dead_factory_note: None,
    };
    let method_views = factory.method_views(&snapshot);
    let method = method_views
        .iter()
        .map(|view| format!("{}:{}", view.method_title, view.method_score))
        .collect::<Vec<_>>()
        .join(",");

    let param_views = snapshot
        .records
        .iter()
        .map(|record| view_record::ParamView {
            param_title: record.raw_label.clone(),
            param_score: record.raw_value,
            dead_param_note: None,
        })
        .collect::<Vec<_>>();
    let param = summarize_param_views(param_views);

    let impl_iter = impl_iter_views(&snapshot)
        .map(|view| format!("{}:{}", view.impl_title, view.impl_score))
        .collect::<Vec<_>>()
        .join(",");

    let dyn_iter = dyn_iter_views(&snapshot)
        .map(|view| format!("{}:{}", view.dyn_title, view.dyn_score))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{titles}:{filtered}:{}:{children}:{local}:{lazy}:{returned}:{method}:{param}:{impl_iter}:{dyn_iter}",
        loop_titles.join(",")
    )
}

fn build_returned_views(snapshot: &source_record::SourceSnapshot) -> Vec<view_record::ReturnedView> {
    snapshot
        .records
        .iter()
        .map(|record| view_record::ReturnedView {
            returned_title: record.raw_label.clone(),
            returned_score: record.raw_value,
            dead_returned_note: None,
        })
        .collect::<Vec<_>>()
}

struct ViewFactory {
    prefix: String,
    dead_factory_note: Option<String>,
}

impl ViewFactory {
    fn method_views(
        &self,
        snapshot: &source_record::SourceSnapshot,
    ) -> Vec<view_record::MethodView> {
        snapshot
            .records
            .iter()
            .map(|record| view_record::MethodView {
                method_title: format!("{}:{}", self.prefix, record.raw_label),
                method_score: record.raw_value,
                dead_method_note: None,
            })
            .collect::<Vec<_>>()
    }
}

fn summarize_param_views(views: Vec<view_record::ParamView>) -> String {
    views
        .iter()
        .map(|view| format!("{}:{}", view.param_title, view.param_score))
        .collect::<Vec<_>>()
        .join(",")
}

fn impl_iter_views(
    snapshot: &source_record::SourceSnapshot,
) -> impl Iterator<Item = view_record::ImplIterView> + '_ {
    snapshot.records.iter().map(|record| view_record::ImplIterView {
        impl_title: record.raw_label.clone(),
        impl_score: record.raw_value,
        dead_impl_note: None,
    })
}

fn dyn_iter_views(
    snapshot: &source_record::SourceSnapshot,
) -> Box<dyn Iterator<Item = view_record::DynIterView> + '_> {
    Box::new(snapshot.records.iter().map(|record| view_record::DynIterView {
        dyn_title: record.raw_label.clone(),
        dyn_score: record.raw_value,
        dead_dyn_note: None,
    }))
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
