#![allow(dead_code)]

use super::common_regex;
use super::regex_utils;
use crate::utils::common;
use regex::Regex;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Stat {
    Count { act: Option<i32>, exp: Option<i32> }, // actual count out of expected count or objective. ex. 2/5 reads "Done 2 out of 5."
    Bool { act: bool, exp: bool }, // -: not done (but implicitly required!), !: done, -/!: explicit default of "-", !/-: done, wasn't required, /-: not required.
    // act is set to false (-) or not done by default if absent. exp is set to true (!) or required if absent.
    RequiredCount { act: i32, exp: bool }, // Like count, except there is no objective count. Only whether it is required or not.
    // 5/- Did five, not required. 0/! Did 0, required!
    // Counts Without an objective have undefined requirement status. That is determined through other elements. This explicitly defines it.
    Unknown, // ? signifies unknown status.
}

impl Stat {
    pub fn from_count(act: Option<i32>, exp: Option<i32>) -> Self {
        Self::Count { act, exp }
    }
    pub fn from_bool(act: bool, exp: bool) -> Self {
        Self::Bool { act, exp }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum StatParseError {
    InvalidMatch, // all-catch
}

pub mod stat_parser_regex {
    use super::*;
    lazy_static! {
        pub static ref COUNT: String = format!(r"((?x) ^ (
            ((?P<ACT0>{INTEGER}) \s* / \s* (?P<EXP0>{INTEGER})) # ACT/EXP
            | (?P<ACT1>{INTEGER})                               # ACT
            | (/ \s* (?P<EXP2>{INTEGER}))                       # /EXP
            )$)", INTEGER=common_regex::INTEGER);
        pub static ref BOOL: String = format!(r"((?x) ^ (
            ((?P<ACT0>[-!]) \s* / \s* (?P<EXP0>[-!]))           # ACT/EXP
            | (?P<ACT1>[-!])                                    # ACT
            | (/ \s* (?P<EXP2>[-!]))                            # /EXP
            )$)");
        pub static ref REQUIRED_COUNT: String = format!(r"((?x) ^ (
            ((?P<ACT>{INTEGER}) \s* / \s* (?P<EXP>[-!]))        # ACT/EXP
            )$)", INTEGER=common_regex::INTEGER);
        pub static ref UNKNOWN: String = format!(r"((?x) ^ \? $)");
        pub static ref STAT: String = regex_utils::filter_inner_capture_group_names(&format!(r"((?x) ^ (
            {COUNT} | {BOOL} | {REQUIRED_COUNT} | {UNKNOWN}
            )$)", COUNT=COUNT.as_str(), BOOL=BOOL.as_str(), REQUIRED_COUNT=REQUIRED_COUNT.as_str(),
            UNKNOWN=UNKNOWN.as_str()));

        // compiled regex
        pub static ref COUNT_RE: Regex = Regex::new(&COUNT).unwrap();
        pub static ref BOOL_RE: Regex = Regex::new(&BOOL).unwrap();
        pub static ref REQUIRED_COUNT_RE: Regex = Regex::new(&REQUIRED_COUNT).unwrap();
        pub static ref UNKNOWN_RE: Regex = Regex::new(&UNKNOWN).unwrap();
        pub static ref STAT_RE: Regex = Regex::new(&STAT).unwrap();
    }
}

impl std::str::FromStr for Stat {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_custom_bool(s: &str) -> Result<bool, ()> {
            match s {
                "!" => Ok(true),
                "-" => Ok(false),
                _ => Err(()),
            }
        }

        if let Some(cap) = stat_parser_regex::COUNT_RE.captures(s) {
            // captures the tokens from the regex depending on which variant they end up on
            let act_cap = cap
                .name("ACT0")
                .or_else(|| cap.name("ACT1"))
                .map(|m| m.as_str());
            let exp_cap = cap
                .name("EXP0")
                .or_else(|| cap.name("EXP2"))
                .map(|m| m.as_str());

            let act = if act_cap.is_some() {
                Some(common::parse_integer_auto(act_cap.unwrap()).unwrap())
            } else {
                None
            };
            let exp = if exp_cap.is_some() {
                Some(common::parse_integer_auto(exp_cap.unwrap()).unwrap())
            } else {
                None
            };
            Ok(Stat::Count { act, exp })
        } else if let Some(cap) = stat_parser_regex::BOOL_RE.captures(s) {
            // captures the tokens from the regex depending on which variant they end up on
            let act_cap = cap
                .name("ACT0")
                .or_else(|| cap.name("ACT1"))
                .map(|m| m.as_str());
            let exp_cap = cap
                .name("EXP0")
                .or_else(|| cap.name("EXP2"))
                .map(|m| m.as_str());
            // actual defaults to false (not done) and expected defaults to true (required to bedone)
            let act = if act_cap.is_some() {
                parse_custom_bool(act_cap.unwrap()).unwrap()
            } else {
                false
            };
            let exp = if exp_cap.is_some() {
                parse_custom_bool(exp_cap.unwrap()).unwrap()
            } else {
                true
            };
            Ok(Stat::Bool { act, exp })
        } else if let Some(cap) = stat_parser_regex::REQUIRED_COUNT_RE.captures(s) {
            // captures the tokens from the regex depending on which variant they end up on
            let act_cap = cap.name("ACT").map(|m| m.as_str());
            let exp_cap = cap.name("EXP").map(|m| m.as_str());

            let act = common::parse_integer_auto(act_cap.unwrap()).unwrap();
            let exp = parse_custom_bool(exp_cap.unwrap()).unwrap();
            Ok(Stat::RequiredCount { act, exp })
        } else if let Some(_cap) = stat_parser_regex::UNKNOWN_RE.captures(s) {
            Ok(Stat::Unknown)
        } else {
            Err(format!("No variant of Stat is satisfied by '{}'", s).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----------------------------
    // Stat Tests -----------------
    // ----------------------------
    
    #[test]
    fn test_stat_parse() {
        fn assert_parses(stat_format: &str, exp_stat: &Stat) {
            println!("test format: {}", stat_format);
            assert_eq!(stat_format.parse::<Stat>().unwrap(), *exp_stat);
        }

        fn assert_fails(invalid_stat_format: &str) {
            println!("test format: {}", invalid_stat_format);
            // you can just not unwrap or use unwrap_err, but showcasing this is cool!
            let res = std::panic::catch_unwind(|| invalid_stat_format.parse::<Stat>().unwrap());
            assert!(res.is_err());
        }

        // assert INTEGER token is functional 
        assert!(common_regex::INTEGER_RE.captures("0").is_some(), "invalid INTEGER token");

        assert_parses("0/5", &Stat::Count {act: Some(0), exp: Some(5)});
        assert_parses("2/020", &Stat::Count {act: Some(2), exp: Some(20)});
        assert_parses("/0xFF", &Stat::Count {act: None, exp: Some(0xFF)});
        assert_parses("0x0/0b0", &Stat::Count {act: Some(0), exp: Some(0)});
        assert_parses("5 /  5", &Stat::Count {act: Some(5), exp: Some(5)});
        assert_parses("5 /5", &Stat::Count {act: Some(5), exp: Some(5)});
        assert_parses("3", &Stat::Count {act: Some(3), exp: None});
        assert_parses("999", &Stat::Count {act: Some(999), exp: None});

        assert_parses("/-", &Stat::Bool {act: false, exp: false});
        assert_parses("-", &Stat::Bool {act: false, exp: true});
        assert_parses("/!", &Stat::Bool {act: false, exp: true});
        assert_parses("!/-", &Stat::Bool {act: true, exp: false});
        assert_parses("!", &Stat::Bool {act: true, exp: true});
        
        assert_parses("0/-", &Stat::RequiredCount {act: 0, exp: false});
        assert_parses("1/!", &Stat::RequiredCount {act: 1, exp: true});
        
        assert_parses("?", &Stat::Unknown);

        assert_fails("Should obviously fail!");
        
        // should not handle untrimmed space
        assert_fails(" 0/5");
        assert_fails("0     ");

        assert_fails("/");
        assert_fails(",999");
        assert_fails("(999)");
        assert_fails("999 // beep boop");
    }
}
