pub fn dead_const_report(raw: &str) -> String {
    const_support::dead_const(raw)
}
