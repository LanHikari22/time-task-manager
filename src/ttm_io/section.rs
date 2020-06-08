//! @author LanHikari22 <lanhikarixx@gmail.com>
//! 
//! responsible for the parsing of Sections from a file stream or string
//! A section always start with a section specifier [Section Pattern] and ends with either the
//! start of a new section at the same tab level, or EOF (or End of Stream. or End of String, or whatever suites you!).
//! 
//! Sections may not contain any meta-data in them, so nested sections will have to be parsed per the payload grammar.

#![allow(dead_code)]

use std::borrow::Cow;
use crate::utils::common::StrUtils;
use crate::utils::scanner;
use scanner::{FromNext, StrScanner};

#[derive(Debug, PartialEq)]
struct Section {
    tab: String,
    pattern: String,
    body: String,
}

#[derive(Debug, PartialEq)]
enum SectionParseError {
    /// error message of parsing error
    Message(Cow<'static, str>),
    /// could not parse entire string, but successfully parsed a substr
    /// returns the remaining substr
    Next(Section, String),
}

impl scanner::FromNext for Section {
    type Err = SectionParseError;
    fn next(s: &str) -> Result<(usize, Self), Self::Err> {
        let line_idx = s.find("\n").or_else(|| Some(s.len())).unwrap();

        let specifier_line_error_msg = "Could not parse specifier line";

        let mut scan = StrScanner::create(s);
        let specifier_line = scan.next_line()
            .or_else(|e| Err(SectionParseError::Message(specifier_line_error_msg.into())))?;
        let trimmed_specifier_line = specifier_line.trim();

        // match "\[.*\]" and extract specifier
        if !trimmed_specifier_line.starts_with("[") || !trimmed_specifier_line.ends_with("]") {
            return Err(SectionParseError::Message(specifier_line_error_msg.into()));
        }
        let specifier = &specifier_line[1..specifier_line.len()-1];
        let specifier_tab = StrUtils(&specifier_line).tabs();

        // parse section specifier pattern
        let open_bracket_idx = specifier_line.find("[").unwrap();
        let closed_bracket_idx = specifier_line.find("]").unwrap();
        let specifier = specifier_line[open_bracket_idx+1..closed_bracket_idx].trim();

        // make sure not to encounter another section, or just parse partially
        // Consume until beginning of new section, end of stream, or end of tab level
        let body_start_cur = scan.cur;
        while let Ok((len, line)) = scan.peek_line() {
            let trimmed_line = line.trim();
            let line_tab = StrUtils(&line).tabs();

            // reached end of tab body
            if !line_tab.is_empty() && line_tab.len() < specifier_tab.len() {break;}

            // start of new section detected
            if line_tab.len() == specifier_tab.len() && trimmed_line.starts_with("[") && trimmed_line.ends_with("]") {
                break;
            }
            
            // consume all other lines
            scan.advance(len);
        }
        let section_body = &scan.stream[body_start_cur..scan.cur];

        Ok((scan.cur, Section {
            tab: specifier_tab.into(),
            pattern: specifier.into(),
            // body: StrUtils(section_body).untab(specifier_tab)
            body: section_body.into(),
        }))
    }
}

impl std::str::FromStr for Section {
    type Err = SectionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (cur, out) = (Self as FromNext).next(s);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_parsing() {
        fn assert_parses(s: &str, exp: &Section) {
            assert_eq!(s.parse::<Section>().unwrap(), *exp);
        }
        assert_parses(
            "[Test Section]\n\
            Some Body here!\n\
            And here too!\
            ".trim(), 
            &Section {
                tab: "".into(),
                pattern: "Test Section".into(),
                body: "Some Body here!\n\
                       And here too!".into(),
            });

        assert_parses(
            "\
                [Logging Section!!]\n\
                Some Body here!\n\
                And here too!\
            ".trim(), 
            &Section {
                tab: "   ".into(),
                pattern: "Logging Section!!".into(),
                body: "     Some Body here!\n\
                            And here too!".into(),
            });
    }
}
