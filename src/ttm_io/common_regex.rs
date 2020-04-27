//! Represents common grammar functions and shared regex
use regex::Regex;
use super::regex_utils;

/// function to experiment with grammar regex
pub fn experiment_parse_var(s: &str) {
    const INT_TYPE_REGEX: &str = r"(u8|u16|u32|i8|i16|i32)";
    const KEYWORD_REGEX: &str = r"([a-zA-Z][a-zA-Z0-9_]*)";
    const LITERAL_INT_REGEX: &str = r"(0x[a-fA-F1-9][a-fA-F0-9]*|[1-9][0-9]*)";
    lazy_static!{
        static ref VARIABLE_DECLERATION_REGEX: String = format!(r"let (?P<VarName>{KEYWORD}): (?P<TypeName>{INT_TYPE}) = (?P<Value>{KEYWORD}|{NUMBER});", 
                                                                KEYWORD=KEYWORD_REGEX, INT_TYPE=INT_TYPE_REGEX, NUMBER=LITERAL_INT_REGEX);
        static ref RE_VARIABLE_DECLERATION: Regex = Regex::new(&regex_utils::filter_inner_capture_group_names(&VARIABLE_DECLERATION_REGEX)).unwrap();
    }
    println!("{}", s);
    println!("{:?}", RE_VARIABLE_DECLERATION.captures(s));
    println!("{:?}", RE_VARIABLE_DECLERATION.captures(s).and_then(|cap| cap.name("VarName").map(|v| v.as_str())));
    println!("{:?}", RE_VARIABLE_DECLERATION.captures(s).and_then(|cap| cap.name("TypeName").map(|v| v.as_str())));
    println!("{:?}", RE_VARIABLE_DECLERATION.captures(s).and_then(|cap| cap.name("Value").map(|v| v.as_str())));
}


/// this defines common regex tokens and compiled regex for elements that can be found across
/// parsers
pub const INTEGER: &str = r"((?x) 0x[a-fA-F0-9]+ | 0o[0-7]+ | 0b[0-1]+ | \d+)";
lazy_static! {
    // compiled regex
    pub static ref INTEGER_RE: Regex = Regex::new(&INTEGER).unwrap();
}


#[cfg(test)]
mod tests {
    #[allow(unused_imports)] use super::*;
}
