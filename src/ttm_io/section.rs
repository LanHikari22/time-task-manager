//! @author LanHikari22
//! 
//! responsible for the parsing of Sections from a file stream or string
//! A section always start with a section specifier [Section Pattern] and ends with either the
//! start of a new section at the same tab level, or EOF (or End of Stream. or End of String, or whatever suites you!).
//! 
//! Sections may not contain any meta-data in them, so nested sections will have to be parsed per the payload grammar.

#![allow(dead_code)]

use std::borrow::Cow;
use crate::common::StrUtils;

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

impl std::str::FromStr for Section {
    type Err = SectionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let line_idx = s.find("\n").or_else(|| Some(s.len())).unwrap();
        let specifier_line = &s[..line_idx];
        let s = &s[line_idx..];

        if !StrUtils(&specifier_line).contains_all("[]")
            {return Err(SectionParseError::Message(
                format!("input str does not start with a section specifier: {}", specifier_line).into()))}
        
        // determine tab level
        // TODO break if tab level is ever broken too
        let specifier_tab = StrUtils(specifier_line).tabs().to_string();

        // parse section specifier pattern
        let open_bracket_idx = specifier_line.find("[").unwrap();
        let closed_bracket_idx = specifier_line.find("]").unwrap();
        let specifier = specifier_line[open_bracket_idx+1..closed_bracket_idx].trim();

        // make sure not to enconter another section, or just parse partially
        let section_start_idx = s.find(&(format!("\n{}[", specifier_tab)));
        if section_start_idx.is_some() {
            let i = section_start_idx.unwrap();
            return Err(SectionParseError::Next (
                Section {
                    tab: specifier_tab, 
                    pattern: specifier.into(), 
                    body: (&s[..i]).trim().into()
                },
                (&s[i..]).into(),))
        }

        Ok(Section {
            tab: specifier_tab,
            pattern: specifier.into(),
            body: s.trim().into()
        })
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
