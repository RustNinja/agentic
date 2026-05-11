use regex_lite::Regex;
use std::sync::LazyLock;

static ROUTE_RE: LazyLock<Regex> = LazyLock::new(|| match Regex::new("/session/") {
    Ok(regex) => regex,
    Err(error) => panic!("invalid route regex: {}", error.message()),
});

pub fn selected_route_id(raw: &str) -> String {
    ROUTE_RE
        .captures(raw)
        .and_then(|captures| captures.name("id"))
        .map(|route_match| route_match.as_str().to_string())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_route_id(raw: &str) -> String {
    Regex::new("/dead/")
        .map(|regex| regex.replace_all(raw, "dead"))
        .unwrap_or_else(|error| error.dead_message())
}
