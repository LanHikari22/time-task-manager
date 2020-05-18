// TODO: remove when this is stable
#![allow(dead_code)]

use std::borrow::Cow;
use regex::Regex;
use super::common_regex;
use super::regex_utils;

pub mod date_regex {
    use super::*;


    lazy_static! {
        pub static ref SHORT_DATE_CODE: String = format!(r"((?x) ^(
            W(?P<Week>{INTEGER}|[A-Fa-f])(?P<Day>[MTWRSU])
        )$)", INTEGER=common_regex::INTEGER);
        pub static ref SHORT_WEEK_DATE_CODE: String = format!(r"((?x) ^(
            W(?P<Week>{INTEGER}|[A-Fa-f])
        )$)", INTEGER=common_regex::INTEGER);
        pub static ref LONG_DATE_CODE: String = format!(r"((?x) ^(
            Y(?P<Year>{INTEGER})(?P<Season>[MFWS])-W(?P<Week>{INTEGER}|[A-Fa-f])(?P<Day>[MTWRSU])
        )$)", INTEGER=common_regex::INTEGER);
        pub static ref LONG_WEEK_DATE_CODE: String = format!(r"((?x) ^(
            Y(?P<Year>{INTEGER})(?P<Season>[MFWS])-W(?P<Week>{INTEGER}|[A-Fa-f])
        )$)", INTEGER=common_regex::INTEGER);

        // compiled regex
        pub static ref SHORT_DATE_CODE_RE: Regex = Regex::new(&SHORT_DATE_CODE).unwrap();
        pub static ref SHORT_WEEK_DATE_CODE_RE: Regex = Regex::new(&SHORT_WEEK_DATE_CODE).unwrap();
        pub static ref LONG_DATE_CODE_RE: Regex = Regex::new(&LONG_DATE_CODE).unwrap();
        pub static ref LONG_WEEK_DATE_CODE_RE: Regex = Regex::new(&LONG_WEEK_DATE_CODE).unwrap();
    }
}


#[derive(PartialEq, Debug)]
pub enum Season {
    Summer, Fall, Winter, Spring,
}

impl std::str::FromStr for Season {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
        "M" => return Ok(Season::Summer),
        "F" => return Ok(Season::Fall),
        "W" => return Ok(Season::Winter),
        "S" => return Ok(Season::Spring),
        _ => return Err(format!("Invalid Season Code: {}", s).into())
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Weekday {
    Mon, Tue, Wed, Thu, Fri, Sat, Sun,
}

impl std::str::FromStr for Weekday {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
        "M" | "Mon" | "Monday"      => return Ok(Weekday::Mon),
        "T" | "Tue" | "Tuesday"     => return Ok(Weekday::Tue),
        "W" | "Wed" | "Wednesday"   => return Ok(Weekday::Wed),
        "R" | "Thu" | "Thursday"    => return Ok(Weekday::Thu),
        "F" | "Fri" | "Friday"      => return Ok(Weekday::Fri),
        "S" | "Sat" | "Saturday"    => return Ok(Weekday::Sat),
        "U" | "Sun" | "Sunday"      => return Ok(Weekday::Sun),
        _ => return Err(format!("Invalid Season Code: {}", s).into())
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Date {
    DateCode {year: u32, season: Season, week: u32, day: Weekday},
    WeekDateCode {year: u32, season: Season, week: u32},
    ShortDateCode {week: u32, day: Weekday},
    ShortWeekDateCode {week: u32},
}

impl std::str::FromStr for Date {
    type Err = Cow<'static, str>;

    /// parses date codes of the form
    /// - W<num><day> like W8T for Week 8 Tuesday
    /// - Y<num><season>-W<num><day> like Y20S-W8M for Year (20)20, Week 8 Monday
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if let Some(cap) = date_regex::SHORT_DATE_CODE_RE.captures(s) {
            return Ok(Date::ShortDateCode {
                week: regex_utils::capture_parse_u32(&cap, "Week"), 
                day: regex_utils::capture_parse(&cap, "Day")});
        } 
        else if let Some(cap) = date_regex::SHORT_WEEK_DATE_CODE_RE.captures(s) {
            return Ok(Date::ShortWeekDateCode {
                week: regex_utils::capture_parse_u32(&cap, "Week")});
        }
        else if let Some(cap) = date_regex::LONG_DATE_CODE_RE.captures(s) {
            return Ok(Date::DateCode {
                year: regex_utils::capture_parse_u32(&cap, "Year"), 
                season: regex_utils::capture_parse(&cap, "Season"), 
                week: regex_utils::capture_parse_u32(&cap, "Week"), 
                day: regex_utils::capture_parse(&cap, "Day")});
        }
        else if let Some(cap) = date_regex::LONG_WEEK_DATE_CODE_RE.captures(s) {
            return Ok(Date::WeekDateCode {
                year: regex_utils::capture_parse_u32(&cap, "Year"), 
                season: regex_utils::capture_parse(&cap, "Season"), 
                week: regex_utils::capture_parse_u32(&cap, "Week")});
        }
        else {return Err(format!("could not parse {} as a DateCode", s).into());}
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        fn assert_parsing(s: &str, act: Date) {
            let exp: Date = s.parse().unwrap();
            assert_eq!(exp, act);
        }

        assert_parsing("W8T", Date::ShortDateCode {week: 8, day: Weekday::Tue});
        assert_parsing("W333", Date::ShortWeekDateCode {week: 333});
        assert_parsing("Y20S-W8M", Date::DateCode {year: 20, season: Season::Spring, week: 8, day: Weekday::Mon});
        assert_parsing("Y22M-W1", Date::WeekDateCode {year: 22, season: Season::Summer, week: 1});
        assert_parsing("Y20M-WF", Date::WeekDateCode {year: 20, season: Season::Summer, week: 0xF});
        assert_parsing("Y20S-WAU", Date::DateCode {year: 20, season: Season::Spring, week: 0xA, day: Weekday::Sun});
    }
}