pub struct VecBoxIterAsRefMapPayload {
    value: String,
}

impl VecBoxIterAsRefMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-box-iter-as-ref-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-box-iter-as-ref-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-box-iter-as-ref-map:{}", self.value)
    }
}

fn vec_box_iter_as_ref_map_payloads(raw: &str) -> Vec<Box<VecBoxIterAsRefMapPayload>> {
    raw.split(',')
        .filter(|part| !part.trim().is_empty())
        .map(|part| Box::new(VecBoxIterAsRefMapPayload::new(part)))
        .collect()
}

pub fn selected_vec_box_iter_as_ref_map(raw: &str) -> String {
    vec_box_iter_as_ref_map_payloads(raw)
        .iter()
        .map(|payload| payload.as_ref().render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_vec_box_iter_as_ref_map(raw: &str) -> String {
    VecBoxIterAsRefMapPayload::new(raw).dead_method()
}
