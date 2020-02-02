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
        scope -= common::str_count_char(scope_token, ')');
        // next token will be in a nested layer
        scope += 1;
    }

    out_regex.join("(")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_inner_capture_group_names() {
        assert_eq!(filter_inner_capture_group_names(r"(?P<VALUE>(?P<NUMBER>[0-9]))"), r"(?P<VALUE>([0-9]))");
        assert_eq!(filter_inner_capture_group_names(r"TEST (?P<TOP> (?P<INNER>SOMETHING))"), r"TEST (?P<TOP> (SOMETHING))");
        assert_eq!(filter_inner_capture_group_names(r"TEST (?P<TOP> (?P<INNER>SOMETHING (?P<INNER2>SOMETHING_ELSE)) (?P<INNER>SOMETHING))"), 
                                                   r"TEST (?P<TOP> (SOMETHING (SOMETHING_ELSE)) (SOMETHING))");
    }
}
