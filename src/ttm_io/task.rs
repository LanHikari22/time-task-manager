// TODO remove when this is in stable development
#![allow(dead_code)]
// #![allow(unused_imports)]
// TODO Tasks:
//  - Rewrite the REGEX implementation to conform to the new Task description at
//  [*ttm_io/tasks/TaskRegex]
//  - Thoroughly test the requirements of the parsing

//! Implementation of the Task token text format.
//! The Task token is described in the following regex: [^ttm_io/tasks/TaskRegex]
//!     [TaskFlags]\([Daystat][, AccStat][, ContextStat]\) TaskName [\([; due: DateCode]\
//!     [; prior: uint][; *Link]\)]
//!         - TaskFlags: Specifies the state of the Task. Refer to struct `TaskFlags.`
//!         - \([Daystat][, AccStat][, ContextStat]\):
//!             - Refer to `super::stat::Stat`. DayStat specifies the count for today,
//!               AccStat specifies the accumulated count for the task's lifetime,
//!               and ContextStat is the sum of the accumulated sum of DayStat and all child
//!               elements of this task.
//!               - For example, "(1,2,10) Solve some mystery" would have 1 block of time performed
//!               today, 2 blocks of time performed previously in this task, and 10 blocks of time
//!               performed in this task and its children (this task being the context).
//!               - AccStat = accum(DayStat);
//!               - ContextStat = accum(AccStat) + accum(child.AccStat) for each child
//!         - TaskName: Arbitrary text. Can include any arbitrary symbols except for parenthesis.
//!         That would count as the suffix descriptor. [^1].
//!         - [; due: Datecode]: When is this Task due? Refer to `super::date::Date.`
//!             - Keep in mind that the ';' is optional if this is the first entry in the task
//!             post-meta.
//!         - [; prior: uint]: Specifies priority of the task. 0-99, where 0 is most important and
//!                           99 is no priority.
//!         - [; *Link]: Links this task to a note.
//!
//! Previous Iterations of Task metainformation
//!     07-Jan-21
//!     - [TaskFlags]\([DayStat][, AccStat]; DoneDateCode; [due|hard: DateCode;] [g[Name]: Stat[,AccStat];] [rept: (D|W)[<N>];]\) TaskName
//!         - Used to include a lot of information prefix to the task, but this really hurt
//!           readability. I also didn't find use for "hard" deadlines, custom stats, or
//!           specifying that a task will repeat as of now
//!
//! Footnotes
//! [^1]: [*ttm_io/tasks/SuffixDescriptor]   Information at the end of the task.

use super::date;
use super::stat;
use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use stat::Stat;

/// implementation of the task token regex. Must conform to the requirements of
/// [*ttm_io/tasks_TaskRegex].
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
                _ => {
                    return Err(format!("could not parse TaskFlag '{}' found in '{}'", c, s).into())
                }
            }
        }

        Ok(out)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskParseError {
    NoTaskDescriptorsFound,
    InvalidPrefixDescriptor(Cow<'static, str>),
    InvalidSuffixDescriptor(Cow<'static, str>),
    InvalidTaskFlags(Cow<'static, str>),
    InvalidGoalStats(Cow<'static, str>),
    InvalidPriorityValue,
    InvalidDueDate(Cow<'static, str>),
    InvalidHardDate(Cow<'static, str>),
    UnsupportedDescriptorKey {
        key: Cow<'static, str>,
        field: Cow<'static, str>,
    },
    InvalidMatch, // all-catch
}

impl fmt::Display for TaskParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let msg: Cow<'static, str> = match self {
            Self::NoTaskDescriptorsFound => "A task should contain Prefix () at a minimum".into(),
            Self::InvalidPrefixDescriptor(message) => message.to_owned(),
            Self::InvalidSuffixDescriptor(message) => message.to_owned(),
            Self::InvalidTaskFlags(message) => message.to_owned(),
            Self::InvalidGoalStats(message) => message.to_owned(),
            Self::InvalidPriorityValue => {
                "Failed to not parse priority as an unsigned integer".into()
            }
            Self::InvalidDueDate(message) => {
                format!("Failed to parse due date descriptor field: {}", message).into()
            }
            Self::InvalidHardDate(message) => {
                format!("Failed to parse hard date descriptor field: {}", message).into()
            }
            Self::UnsupportedDescriptorKey { key, field } => {
                format!("unsupported keyword argument {} in {}", key, field).into()
            }
            Self::InvalidMatch => "Unknown Error".into(),
        };
        write!(f, "{}", msg)?;
        Ok(())
    }
}

#[derive(PartialEq, Debug)]
pub struct Task {
    /// describes the current state of the task
    flags: TaskFlags,
    /// how many units of time were accomplished in this task today
    day_stat: Option<Stat>,
    /// how many units of time total were accomplished
    accum_stat: Option<Stat>,
    /// how many units of time were accomplished in this task and its children
    context_stat: Option<Stat>,
    /// describes the task to be accomplished
    name: String,
    /// a pattern that could be searched to a note
    note_link: String,
    /// allows tasks to be sorted by importance
    priority: usize,
    /// the task's deadline, when it is expected to be done
    due_date: Option<date::Date>,
    /// hard deadline, could be bad to miss
    hard_date: Option<date::Date>,
    /// custom counters used in the task to track progress
    other_stats: HashMap<String, [Option<Stat>; 3]>,
}

impl Task {
    /// maximum value allowed for task priority. Lower is more important.
    const NO_PRIORITY: usize = 99;

    fn from_name_and_stats(name: &str, stats: (Option<Stat>, Option<Stat>, Option<Stat>)) -> Self {
        Self {
            flags: TaskFlags::empty(),
            day_stat: stats.0,
            accum_stat: stats.1,
            context_stat: stats.2,
            name: name.to_owned(),
            note_link: "".to_string(),
            priority: Task::NO_PRIORITY,
            due_date: None,
            hard_date: None,
            other_stats: HashMap::new(),
        }
    }

    fn from_name(name: &str) -> Self {
        Self::from_name_and_stats(name, (None, None, None))
    }

    fn build(self) -> Self {
        self
    }

    fn build_flags(&mut self, flags: TaskFlags) -> &mut Self {
        self.flags = flags;
        self
    }

    fn build_note_link(&mut self, note_link: &str) -> &mut Self {
        self.note_link = note_link.to_string();
        self
    }

    fn build_priority(&mut self, priority: usize) -> &mut Self {
        if priority > Task::NO_PRIORITY {
            panic!(format!(
                "Task Priority cannot exceed max value of {}",
                Task::NO_PRIORITY
            ));
        }
        self.priority = priority;
        self
    }

    fn build_due_date(&mut self, due_date: date::Date) -> &mut Self {
        self.due_date = Some(due_date);
        self
    }

    fn build_hard_date(&mut self, hard_date: date::Date) -> &mut Self {
        self.hard_date = Some(hard_date);
        self
    }

    fn build_other_stat(&mut self, goal: &str, stats: [Option<Stat>; 3]) -> &mut Self {
        self.other_stats.insert(goal.to_string(), stats);
        self
    }
}

impl std::str::FromStr for Task {
    type Err = TaskParseError;

    /// parses a Task entry of the form specified in [*ttm_io/tasks/TaskRegex]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // identify prefix descriptor range
        let idx_prefix_open_paren: Option<usize> = s.find('(');
        let idx_prefix_closed_paren: Option<usize> = s.find(')');
        if !(idx_prefix_open_paren.is_some() && idx_prefix_closed_paren.is_some()) {
            // All valid tasks must contain metainformation, even if it's empty
            return Err(TaskParseError::NoTaskDescriptorsFound);
        }
        let idx_prefix_open_paren: usize = idx_prefix_open_paren.unwrap();
        let idx_prefix_closed_paren: usize = idx_prefix_closed_paren.unwrap();

        let s_after_prefix: &str = &s[idx_prefix_closed_paren + 1..].trim();

        // identify suffix descriptor range, if found
        let idx_suffix_open_paren: Option<usize> = s_after_prefix.find('(');
        let idx_suffix_close_paren: Option<usize> = s_after_prefix.find(')');

        if !(idx_suffix_open_paren.is_some() && idx_suffix_close_paren.is_some())
            && !(idx_suffix_open_paren.is_none() && idx_suffix_close_paren.is_none())
        {
            // parenthesis are not put correctly for suffix descriptor
            return Err(TaskParseError::InvalidSuffixDescriptor(
                "Check the parenthesis at the end of the Task".into(),
            ));
        }

        // parse the name of the task
        let task_name: &str = match idx_suffix_open_paren {
            Some(idx) => &s_after_prefix[..idx].trim(),
            None => s_after_prefix,
        };

        // create a task state to populate
        let mut res = Task::from_name(task_name);

        // parse task flags
        let task_flags: TaskFlags = match (&s[..idx_prefix_open_paren]).parse() {
            Ok(res) => res,
            Err(e) => return Err(TaskParseError::InvalidTaskFlags(e)),
        };
        res.flags = task_flags;

        // parse prefix descriptor
        let prefix_stats =
            match parse_stat_tuple(&s[idx_prefix_open_paren + 1..idx_prefix_closed_paren]) {
                Ok(stats) => stats,
                Err(msg) => return Err(TaskParseError::InvalidPrefixDescriptor(msg)),
            };
        res.day_stat = prefix_stats[0];
        res.accum_stat = prefix_stats[1];
        res.context_stat = prefix_stats[2];

        // parse suffix descriptor
        if idx_suffix_open_paren.is_some() {
            let idx_open_paren = idx_suffix_open_paren.unwrap();
            let idx_closed_paren = idx_suffix_close_paren.unwrap();

            let fields: Vec<String> =
                match parse_tuple_arguments(&s_after_prefix[idx_open_paren..=idx_closed_paren]) {
                    Ok(_res) => _res,
                    Err(msg) => return Err(TaskParseError::InvalidSuffixDescriptor(msg.into())),
                };

            for field in fields.iter() {
                let idx_colon = field.find(':');
                if idx_colon == None {
                    // empty fields are possible as ordered_arguments used to exist here
                    if field == "" {
                        continue;
                    }

                    if field.starts_with('*') {
                        // parse note link, skip '*'
                        res.note_link = (&field[1..]).to_string();
                    }
                } else {
                    // "key: value" fields
                    let key = (&field[..idx_colon.unwrap()]).to_string();
                    let val = (&field[idx_colon.unwrap() + 1..]).to_string();
                    // println!("key {}, val {}", key, val);

                    if key == "due" {
                        res.due_date = Some(match val.parse::<date::Date>() {
                            Ok(date) => date,
                            Err(msg) => return Err(TaskParseError::InvalidDueDate(msg)),
                        });
                    } else if key == "hard" {
                        res.hard_date = Some(match val.parse::<date::Date>() {
                            Ok(date) => date,
                            Err(msg) => return Err(TaskParseError::InvalidHardDate(msg)),
                        });
                    } else if key == "prior" {
                        res.priority = match val.parse::<usize>() {
                            Ok(prior) => prior,
                            Err(_e) => return Err(TaskParseError::InvalidPriorityValue),
                        };
                    } else if key == "rept" {
                        unimplemented!("rept kwarg not supported yet");
                    } else if key.starts_with("g") {
                        let other_stats = match parse_stat_tuple(&val.trim()) {
                            Ok(stats) => stats,
                            Err(msg) => return Err(TaskParseError::InvalidGoalStats(msg)),
                        };

                        // add to hash map, discard 'g' from key
                        res.other_stats.insert(key[1..].to_owned(), other_stats);
                    } else {
                        return Err(TaskParseError::UnsupportedDescriptorKey {
                            key: key.into(),
                            field: field.to_owned().into(),
                        });
                    }
                }
            }
        }

        Ok(res)
    }
}

/// takes a string of the format (A; B; C; ...; K1: V1; K2; V2)
/// and extracts it into a list of the diffrent fields [A, B, C, K1: V1, K2: V2]
fn parse_tuple_arguments(tup: &str) -> Result<Vec<String>, &'static str> {
    #[derive(PartialEq, Debug)]
    enum State {
        OpenParen,
        Field,
        ClosedParen,
    };
    let mut state: State = State::OpenParen;

    let tokens = tup.split(";");
    let mut out: Vec<String> = vec![];

    for (i, token) in tokens.enumerate() {
        if i == 0 {
            assert_eq!(state, State::OpenParen);
            if !token.contains('(') {
                return Err("no open parenthesis found");
            }
            state = State::Field;
        }
        if token.contains(')') {
            assert_ne!(state, State::ClosedParen);
            state = State::ClosedParen;
        }

        out.push(
            token
                .chars()
                .filter(|&c| c != '(' && c != ')' && c != ';')
                .collect::<String>()
                .trim()
                .to_string(),
        );
    }

    if state != State::ClosedParen {
        return Err("no close parenthesis found");
    }
    Ok(out)
}

/// parses two stat pairs where one or both could be missing
/// @param s    in the format of "[Stat][,Stat][,Stat]"
///
/// # Examples
///
/// ```
/// assert_eq!(parse_stat_tuple("0,0,0", Ok((
///     Some(&Stat::Count {act: Some(0), exp: None}),
///     Some(&Stat::Count {act: Some(0), exp: None}),
///     Some(&Stat::Count {act: Some(0), exp: None}),
/// )));
/// assert_eq!(parse_stat_tuple("0/15", Ok((
///     Some(&Stat::Count {act: Some(0), exp: Some(15)}),
///     None,
///     None,
/// )));
/// assert_eq!(parse_stat_tuple("", Ok((
///     None,
///     None,
///     None,
/// )));
/// ```
///
fn parse_stat_tuple(s: &str) -> Result<[Option<Stat>; 3], Cow<'static, str>> {
    let mut stats: [Option<Stat>; 3] = [None; 3];

    for (i, token) in s.split(",").enumerate() {
        let token = token.trim();

        // if no tokens were found, it iterates over "" once
        if token == "" {
            continue;
        }

        if i > 2 {
            // tuple should be of max size 3
            return Err("Exceeded number of Stats allowed".into());
        }
        stats[i] = match token.parse() {
            Ok(stat) => Some(stat),
            Err(msg) => return Err(msg),
        };
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test;

    #[test]
    fn test_parse_no_suffix_tasks() {
        // This tests the following formats:
        // FAIL Task
        // PASS () Task
        // PASS (0) Task
        // PASS (0,1) Task
        // PASS (0,1,2) Task
        // FAIL (0,1,2,3) Task
        // PASS (-,-,-) Task
        // FAIL (-,-,-,-) Task

        assert_parses_as(
            "() no suffix Task 0",
            &Task::from_name_and_stats("no suffix Task 0", (None, None, None)),
        );
        assert_parses_as(
            "(0) no suffix Task 1",
            &Task::from_name_and_stats("no suffix Task 1", (Some(Stat::from_count(Some(0), None)), None, None)),
        );
        assert_parses_as(
            "(0,1) no suffix Task 2",
            &Task::from_name_and_stats(
                "no suffix Task 2",
                (
                    Some(Stat::from_count(Some(0), None)),
                    Some(Stat::from_count(Some(1), None)),
                    None,
                ),
            ),
        );
        assert_parses_as(
            "(0,1,2) no suffix Task 3",
            &Task::from_name_and_stats(
                "no suffix Task 3",
                (
                    Some(Stat::from_count(Some(0), None)),
                    Some(Stat::from_count(Some(1), None)),
                    Some(Stat::from_count(Some(2), None)),
                ),
            ),
        );
        assert_fails_to_parse_as(
            "(0,1,2,3) no suffix Task 4",
            &TaskParseError::InvalidPrefixDescriptor("".into()),
        );
        assert_parses_as(
            "(-,-,-) no suffix Task 5",
            &Task::from_name_and_stats(
                "no suffix Task 5",
                (
                    Some(Stat::from_bool(false, true)),
                    Some(Stat::from_bool(false, true)),
                    Some(Stat::from_bool(false, true)),
                ),
            ),
        );
        assert_fails_to_parse_as(
            "(-,-,-,-) no suffix Task 6",
            &TaskParseError::InvalidPrefixDescriptor("".into()),
        );
    }

    #[test]
    fn test_parse_task_flags() {
        assert_parses_as(
            ">B(2/15) Current Blocked Task",
            Task::from_name_and_stats(
                "Current Blocked Task",
                (Some(Stat::from_count(Some(2), Some(15))), None, None),
            )
            .build_flags(TaskFlags::CURRENT | TaskFlags::BLOCKED),
        );
        assert_parses_as(
            "~(,,11/12) Complete!",
            Task::from_name_and_stats(
                "Complete!",
                (None, None, Some(Stat::from_count(Some(11), Some(12)))),
            )
            .build_flags(TaskFlags::DONE),
        );
        assert_parses_as(
            "~~LL() Very Complete... Very Late!",
            Task::from_name_and_stats("Very Complete... Very Late!", (None, None, None))
                .build_flags(TaskFlags::DONE | TaskFlags::LATE),
        );
    }

    fn test_parse_suffix_deadlines() {
        assert_parses_as(
            "~~LL() Very Complete... Very Late! (due: Y21W-W3U)",
            Task::from_name_and_stats("Very Complete... Very Late!", (None, None, None))
                .build_flags(TaskFlags::DONE | TaskFlags::LATE)
                .build_due_date("Y21W-W3U".parse().unwrap()),
        );
        assert_parses_as(
            "~~LL() Very Complete... Very Late! (due: Y21W-W3U; hard: Y21W-W4T)",
            Task::from_name_and_stats("Very Complete... Very Late!", (None, None, None))
                .build_flags(TaskFlags::DONE | TaskFlags::LATE)
                .build_due_date("Y21W-W3U".parse().unwrap())
                .build_hard_date("Y21W-W4T".parse().unwrap()),
        );
    }

    #[test]
    fn test_parse_general() {
        assert_parses_as(
            ">(2/15) My Exercise Task! (*P[My note!]; gPushups: 0/10; due: W2M; gPlancks: -,0,0;)",
            Task::from_name_and_stats(
                "My Exercise Task!",
                (Some(Stat::from_count(Some(2), Some(15))), None, None),
            )
            .build_flags(TaskFlags::CURRENT)
            .build_note_link("P[My note!]")
            .build_other_stat(
                "Pushups",
                [Some(Stat::from_count(Some(0), Some(10))), None, None],
            )
            .build_due_date("W2M".parse().unwrap())
            .build_other_stat(
                "Plancks",
                [
                    Some(Stat::from_bool(false, true)),
                    Some(Stat::from_count(Some(0), None)),
                    Some(Stat::from_count(Some(0), None)),
                ],
            ),
        );
    }

    #[test]
    fn test_fn_parse_stat_pair() {
        assert_eq!(
            parse_stat_tuple("0/0").unwrap(),
            [
                Some(Stat::Count {
                    act: Some(0),
                    exp: Some(0)
                }),
                None,
                None,
            ]
        );
        assert_eq!(
            parse_stat_tuple("0,0").unwrap(),
            [
                Some(Stat::Count {
                    act: Some(0),
                    exp: None
                }),
                Some(Stat::Count {
                    act: Some(0),
                    exp: None
                }),
                None,
            ]
        );
        assert_eq!(
            parse_stat_tuple("1 /2 , 333/111").unwrap(),
            [
                Some(Stat::Count {
                    act: Some(1),
                    exp: Some(2)
                }),
                Some(Stat::Count {
                    act: Some(333),
                    exp: Some(111)
                }),
                None
            ]
        );
    }

    #[test]
    fn test_fn_parse_tuple_arguments() {
        assert_eq!(
            parse_tuple_arguments("(A; B; C)"),
            Ok(vec!["A".to_string(), "B".to_string(), "C".to_string()])
        );
        assert_eq!(
            parse_tuple_arguments("(A;;C)"),
            Ok(vec!["A".to_string(), "".to_string(), "C".to_string()])
        );
        assert_eq!(
            parse_tuple_arguments("(;;C)"),
            Ok(vec!["".to_string(), "".to_string(), "C".to_string()])
        );
        assert_eq!(
            parse_tuple_arguments("(ABCabc012!@#; Key: Val)"),
            Ok(vec!["ABCabc012!@#".to_string(), "Key: Val".to_string()])
        );
        assert_eq!(parse_tuple_arguments("(A)"), Ok(vec!["A".to_string()]));
        assert_eq!(parse_tuple_arguments("()"), Ok(vec!["".to_string()]));
        assert_eq!(
            parse_tuple_arguments("(;)"),
            Ok(vec!["".to_string(), "".to_string()])
        );
    }

    fn assert_parses_as(s: &str, exp: &Task) {
        test::assert_parses_as::<Task, TaskParseError>(s, exp);
    }

    fn assert_fails_to_parse_as(s: &str, err: &TaskParseError) {
        let act_err: TaskParseError = s.parse::<Task>().unwrap_err();

        test::assert_variant_eq(&act_err, &err);
    }

    fn assert_parses_mult_as(_s: &str, _exp: &[Task]) {
        unimplemented!();
    }
}
