use regex_lite::{dead_regex_debug, DeadRegex};

pub fn dead_route_id(raw: &str) -> String {
    let regex = DeadRegex::new(raw);
    format!("{}:{}", dead_regex_debug(raw), regex.render())
}
