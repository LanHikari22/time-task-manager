//! regex_utils
//! utility functions for regex handling with things such as grammars

use crate::common;

/// this removes out inner named groups in regex strings so that when the regex is used for 
/// tokenization, only the top-level tokens are considered.
pub fn filter_inner_capture_group_names(regex: &str) -> String {
    let mut scope = 0;
    let mut out_regex: Vec<String> = vec![];
    for scope_token in regex.split(|c| c == '(') {
        // only sdope 1 ?P<...> are allowed -- the rest are filtered
        if scope != 1 && scope_token.starts_with("?P<") {
            let capture_name_start_idx = scope_token.find("?P<").unwrap();
            let capture_name_end_idx = scope_token[capture_name_start_idx..].find(">").unwrap();
            out_regex.push(vec![&scope_token[..capture_name_start_idx], &scope_token[capture_name_end_idx+1..]].join(""));
        } else {
            out_regex.push(scope_token.to_string());
        }

        // acount for descoping
        scope -= common::StrUtils(scope_token).count_char(')');
        // next token will be in a nested layer
        scope += 1;
    }

    out_regex.join("(")
}


/// Handles the logic of parsing a field that has been regex captured
/// fields may be specified in regex with the (?P<Field>) syntax
/// Parses an i32 automatically out of the captured field
/// 
/// # panics
/// Assumes that the field exists and can me captured by cap
/// Assumes that the capture is well-defined and parses successfully
pub fn capture_parse_i32(cap: &regex::Captures<'_>, field: &str) -> i32 {
    let out = cap.name(field).map(|m| m.as_str()).unwrap();
    println!("{}", out);
    let out = common::parse_integer_auto(out).unwrap();

    out
}


/// Handles the logic of parsing a field that has been regex captured
/// fields may be specified in regex with the (?P<Field>) syntax
/// Parses a u32 automatically out of a field
/// 
/// # panics
/// Assumes that the field exists and can me captured by cap
/// Assumes that the capture is well-defined and parses successfully
pub fn capture_parse_u32(cap: &regex::Captures<'_>, field: &str) -> u32 {
    capture_parse_i32(cap, field) as u32
}


/// Handles the logic of parsing a field that has been regex captured
/// fields may be specified in regex with the (?P<Field>) syntax
/// Once captured, this uses T's parse logic to parse T.
/// 
/// # panics
/// Assumes that the field exists and can me captured by cap
/// Assumes that the capture is well-defined and parses successfully
pub fn capture_parse<T>(cap: &regex::Captures<'_>, field: &str) -> T 
    where T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    cap.name(field).map(|m| m.as_str()).unwrap()
        .parse::<T>().unwrap()
}


#[cfg(test)]
mod tests {
    #[allow(unused_imports)] use super::*;

    #[test]
    fn test_filter_inner_capture_group_names() {
        assert_eq!(filter_inner_capture_group_names(r"(?P<VALUE>(?P<NUMBER>[0-9]))"), r"(?P<VALUE>([0-9]))");
        assert_eq!(filter_inner_capture_group_names(r"TEST (?P<TOP> (?P<INNER>SOMETHING))"), r"TEST (?P<TOP> (SOMETHING))");
        assert_eq!(filter_inner_capture_group_names(r"TEST (?P<TOP> (?P<INNER>SOMETHING (?P<INNER2>SOMETHING_ELSE)) (?P<INNER>SOMETHING))"), 
                                                   r"TEST (?P<TOP> (SOMETHING (SOMETHING_ELSE)) (SOMETHING))");
    }
}
