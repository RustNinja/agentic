pub fn dead_macro_report(raw: &str) -> String {
    macro_support::dead_generated(raw)
}
