pub struct IteratorScanStatefulMapPayload {
    value: String,
}

impl IteratorScanStatefulMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-scan-stateful-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-scan-stateful-map:{}", self.value)
    }
}

pub fn selected_iterator_scan_stateful_map(raw: &str) -> String {
    raw.split(',')
        .scan(0usize, |count, part| {
            *count += 1;
            let tagged = format!("{count}:{part}");
            Some(IteratorScanStatefulMapPayload::new(&tagged).render_label())
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_scan_stateful_map(raw: &str) -> String {
    IteratorScanStatefulMapPayload::new(raw).unused_label()
}
