#[macro_export]
macro_rules! expose_registered_field {
    ($ty:ident) => {
        pub fn registered_reason() -> &'static str {
            let field = $ty {
                type_name: "ExperimentalField",
                field_name: "reason",
                reason: "macro-retained",
            };
            let _ = field.type_name;
            let _ = field.field_name;
            field.reason
        }
    };
}
