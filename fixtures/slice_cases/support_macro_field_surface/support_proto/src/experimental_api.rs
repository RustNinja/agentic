pub struct ExperimentalField {
    pub type_name: &'static str,
    pub field_name: &'static str,
    pub reason: &'static str,
}

macro_host::expose_registered_field!(ExperimentalField);
