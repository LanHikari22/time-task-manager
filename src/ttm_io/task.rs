// TODO remove when this is in stable development
#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::HashMap;
use std::borrow::Cow;
use regex::Regex;
use crate::common;
use super::common_regex;
use super::regex_utils;
use super::stat;
use super::date;

use stat::Stat;

pub mod task_parser_regex {
    use super::*;
    lazy_static! {
        pub static ref STAT_PAIR: String = format!(r"((?x) ^ (      # [Stat][,Stat]
            ((?P<Left0> {STAT}))
            | (,(?P<Right1> {STAT}))
            | ((?P<Left2> {STAT})\s* ,(?P<Right2> {STAT}))
        )$)", STAT=stat::stat_parser_regex::STAT.as_str());
        pub static ref TASK: String = format!(r"((?x) ^ (
            (?P<EntryFlags> [>~BL0-9])                              # Flags about the task
            \(
            (?P<DayAccStats> {STAT_PAIR})                           # [DayStat][,AccStat]
            (; (?P<DoneDateCode> {DATE_CODE} )?                     # [; DoneDateCode]
            (; (?P<KeyValuePairs> ((due|hard): {DATE_CODE})*) )?    # [; key:val]*
            (; (?P<KeyValuePairs> (\w+: .*)*) )?                    # [; key:val]*
            ;?\)
            (?P<Name> .*)
        )$)", /*STAT=stat::stat_parser_regex::STAT.as_str(),*/
              DATE_CODE=r".*",
              STAT_PAIR=STAT_PAIR.as_str(),
        );

        // compiled regex
        pub static ref TASK_RE: Regex = Regex::new(&TASK).unwrap();
        pub static ref STAT_PAIR_RE: Regex = Regex::new(&STAT_PAIR).unwrap();
    }
}


bitflags! {
    struct TaskFlags: u32 {
        const BLOCKED = 0b00000001;
        const CURRENT = 0b00000010;
        const LATE    = 0b00000011;
        const DONE    = 0b00000100;
    }
}

impl std::str::FromStr for TaskFlags {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let mut out = Self::empty();
        for c in s.chars() {
            match c {
            'B' => out.insert(TaskFlags::BLOCKED),
            '>' => out.insert(TaskFlags::CURRENT),
            'L' => out.insert(TaskFlags::LATE),
            '~' => out.insert(TaskFlags::DONE),
            _ => return Err(format!("could not parse TaskFlag '{}' found in '{}'", c, s).into()),
            }
        }

        Ok(out)
    }
}

#[derive(Debug)]
pub enum TaskParseError {
    InvalidMatch, // all-catch
}

#[derive(PartialEq, Debug)]
pub struct Task {
    name: String,
    day_stats: (Option<Stat>, Option<Stat>),
    flags: TaskFlags,
    other_stats: HashMap<String, (Option<Stat>, Option<Stat>)>,
    done_date: Option<date::Date>,    
    due_date: Option<date::Date>,
    hard_date: Option<date::Date>,
}

impl Task {
    fn from_name_and_day_stats(name: &str, day_stats: (Option<Stat>, Option<Stat>)) -> Result<Task, Cow<'static, str>> {
        Ok(Task {name: name.to_owned(), day_stats, flags: TaskFlags::empty(), 
            other_stats: HashMap::new(), done_date: None, due_date: None, hard_date: None})
    }

    fn from_name(name: &str) -> Result<Task, Cow<'static, str>> {
        Task::from_name_and_day_stats(name, (None, None))
    }
}

impl std::str::FromStr for Task {
    type Err = Cow<'static, str>;

    /// parses a Task entry of the :write!form
    // [TaskFlags]\([DayStat][, AccStat]; DoneDateCode; [due|hard: DateCode;] [g[Name]: Stat[,AccStat];] [rept: (D|W)[<N>];]\) TaskName
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !(s.contains('(') && s.contains(')')) {return Err("no task descriptors found".into());}
        let open_paren_idx = s.find('(').unwrap();
        let closed_paren_idx = s.find(')').unwrap();

        let mut res = Task::from_name(&s[closed_paren_idx+1..].trim())?;
        res.flags = (&s[..open_paren_idx]).parse::<TaskFlags>()?;

        let mut ordered_arguments = true;
        let fields = parse_tuple_arguments(&s[open_paren_idx..=closed_paren_idx])?;
        for (i, field) in fields.iter().enumerate() {
            let idx = field.find(':');
            if  idx == None {
                if field == "" {continue;}
                if !ordered_arguments {return Err(format!("encountered ordered argument {} after kwarg", field).into());}

                println!("arg {}: {}", i, field);

                match i {
                0 => { // [DayStat][, AccStat]
                    res.day_stats = parse_stat_pair(field)?;
                },
                1 => { // DoneDateCode
                    res.done_date = Some(field.parse::<date::Date>()?)
                },
                _ => {return Err(format!("invalid argument {} at pos {}: exceeded maximum ordered args", field, i).into())}
                }
            } else {
                if ordered_arguments {ordered_arguments = false;}
                let key = &field[..idx.unwrap()];
                let val = &field[idx.unwrap()+1..];
                println!("key {}, val {}", key, val);
                
                if key == "due" {
                    res.due_date = Some(val.parse::<date::Date>()?)
                } else if key == "hard" {
                    res.hard_date = Some(val.parse::<date::Date>()?)
                } else if key == "rept" {
                    unimplemented!("rept kwarg not supported yet");
                } else if key.starts_with("g") {
                    // if res.other_stats.is_none() {res.other_stats = Some(HashMap::new());}
                    res.other_stats.insert(key[1..].to_owned(), parse_stat_pair(&val.trim())?);
                } else {return Err(format!("unsupported keyword argument {} in {}", key, field).into())}
            }
        }

        Ok(res)
    }

}


/// takes a string of the format (A; B; C; ...; K1: V1; K2; V2)
/// and extracts it into a list of the diffrent fields [A, B, C, K1: V1, K2: V2]
fn parse_tuple_arguments(tup: &str) -> Result<Vec<String>, &'static str> {
    #[derive(PartialEq, Debug)]
    enum State {OpenParen, Field, ClosedParen};
    let mut state: State = State::OpenParen;

    let tokens = tup.split(";");
    let mut out: Vec<String> = vec![];

    for (i, token) in tokens.enumerate() {
        if i == 0 {
            assert_eq!(state, State::OpenParen);
            if !token.contains('(') {return Err("no open parenthesis found")}
            state = State::Field;
        }
        if token.contains(')') {
            assert_ne!(state, State::ClosedParen);
            state = State::ClosedParen;
        }

        out.push(token.chars()
            .filter(|&c| c != '(' && c != ')' && c != ';')
            .collect::<String>()
            .trim().to_string()
        );
    }

    if state != State::ClosedParen {return Err("no close parenthesis found")}
    Ok(out)
}

/// parses two stat pairs where one or both could be missing
/// :param s: in the format of [Stat][,Stat]
fn parse_stat_pair(s: &str) -> Result<(Option<Stat>, Option<Stat>), Cow<'static, str>> {
    if s == "" { return Ok((None, None)) } // unspecified s

    if let Some(idx) = s.find(",") {
        let left: &str = &s[..idx].trim();
        let right: &str = &s[idx+1..].trim();

        let left_stat = if left != "" {Some(left.parse::<Stat>()?)} else {None};
        let right_stat = if right != "" {Some(right.parse::<Stat>()?)} else {None};

        return Ok((left_stat, right_stat));
    } else {
        let left_stat = Some(s.parse::<Stat>()?);

        return Ok((left_stat, None));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_task_parsing() {
        fn assert_parses(s: &str, exp: Task) {
            let act = s.parse::<Task>().unwrap();
            assert_eq!(act, exp);
        }

        assert_parses("(0,0) Basic Task", 
            Task::from_name_and_day_stats("Basic Task", 
                (Some(Stat::Count {act: Some(0), exp: None}), 
                 Some(Stat::Count {act: Some(0), exp: None}))).unwrap());
        assert_parses(">B(2/15) Current Blocked Task", 
            Task {
                day_stats:
                    (Some(Stat::Count {act: Some(2), exp: Some(15)}), 
                     None),
                name: "Current Blocked Task".to_owned(),
                flags: TaskFlags::CURRENT | TaskFlags::BLOCKED,
                other_stats: HashMap::new(),
                done_date: None, due_date: None, hard_date: None
            }
        );
        assert_parses("~~LL(-; Y20S-W8W) Completed Late Task", 
            Task {
                day_stats:
                    (Some(Stat::Bool {act: false, exp: true}),
                     None),
                name: "Completed Late Task".to_owned(),
                flags: TaskFlags::DONE | TaskFlags::LATE,
                other_stats: HashMap::new(),
                done_date: Some(date::Date::DateCode {year: 20, season: date::Season::Spring, week: 8, day: date::Weekday::Wed}), 
                due_date: None, hard_date: None
            }
        );
        assert_parses("(due: Y20S-WAU; hard: WC) Important Deadline", 
            Task {
                day_stats: (None, None),
                name: "Important Deadline".to_owned(),
                flags: TaskFlags::empty(),
                other_stats: HashMap::new(),
                done_date: None,
                due_date: Some(date::Date::DateCode {year: 20, season: date::Season::Spring, week: 0xA, day: date::Weekday::Sun}), 
                hard_date: Some(date::Date::ShortWeekDateCode {week: 0xC}), 
            }
        );
        assert_parses(">(2/15; gPushups: 0/10; gPlancks: -;) My Exercise Task!", 
            Task {
                day_stats:
                    (Some(Stat::Count {act: Some(2), exp: Some(15)}), 
                     None),
                name: "My Exercise Task!".to_owned(),
                flags: TaskFlags::CURRENT,
                other_stats: {
                    let mut map = HashMap::new();
                    map.insert("Pushups".into(), (Some(Stat::Count {act: Some(0), exp: Some(10)}), None));
                    map.insert("Plancks".into(), (Some(Stat::Bool {act: false, exp: true}), None));
                    map
                },
                done_date: None, due_date: None, hard_date: None
            }
        );
    }
    
    #[test]
    fn test_parse_tuple_arguments() {
        assert_eq!(parse_tuple_arguments("(A; B; C)"), Ok(vec!["A".to_string(), "B".to_string(), "C".to_string()]));
        assert_eq!(parse_tuple_arguments("(A;;C)"), Ok(vec!["A".to_string(), "".to_string(), "C".to_string()]));
        assert_eq!(parse_tuple_arguments("(;;C)"), Ok(vec!["".to_string(), "".to_string(), "C".to_string()]));
        assert_eq!(parse_tuple_arguments("(ABCabc012!@#; Key: Val)"), Ok(vec!["ABCabc012!@#".to_string(), "Key: Val".to_string()]));
        assert_eq!(parse_tuple_arguments("(A)"), Ok(vec!["A".to_string()]));
        assert_eq!(parse_tuple_arguments("()"), Ok(vec!["".to_string()]));
        assert_eq!(parse_tuple_arguments("(;)"), Ok(vec!["".to_string(), "".to_string()]));
    }

    #[test]
    fn test_parse_stat_pair() {
        assert_eq!(parse_stat_pair("0/0").unwrap(), 
                (Some(Stat::Count {act: Some(0), exp: Some(0)}), 
                 None));
        assert_eq!(parse_stat_pair("0,0").unwrap(), 
                (Some(Stat::Count {act: Some(0), exp: None}), 
                 Some(Stat::Count {act: Some(0), exp: None})));
        assert_eq!(parse_stat_pair("1 /2 , 333/111").unwrap(), 
                (Some(Stat::Count {act: Some(1),   exp: Some(2)}), 
                 Some(Stat::Count {act: Some(333), exp: Some(111)})));

    }
}
