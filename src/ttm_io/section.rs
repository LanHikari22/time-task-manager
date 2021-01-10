//! @author Mohammed Alzakariya (github: LanHikari22) <lanhikarixx@gmail.com>
//!
//! responsible for the parsing of Sections from a file stream or string
//! A section always start with a section specifier [Section Pattern] and ends with either the
//! start of a new section at the same tab level, or EOF (or End of Stream. or End of String, or whatever suites you!).
//!
//! Sections may not contain any meta-data in them, so nested sections will have to be parsed per the payload grammar.

#![allow(dead_code)]

use crate::utils::common::StrUtils;
use crate::utils::scanner;
use scanner::{FromNext, StrScanner};
use std::borrow::Cow;

/// A section comes in the form of
/// |-tab-| [Section Specifier]
/// |-tab-| Content...
///
#[derive(Debug, PartialEq)]
struct Section {
    /// initial tabbing found in the Specifier line
    tab: String,
    specifier: String,
    body: String,
}

#[derive(Debug, PartialEq)]
enum SectionParseError {
    /// error message of generic parsing error
    /// meant to be reported for diagnostics, not handled
    Generic(Cow<'static, str>),
    /// The first line is expected to be a Specifier [Specifier], used to denote
    /// section function
    InvalidSpecifier,
}

impl std::fmt::Display for SectionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let msg: Cow<'static, str> = match self {
            Self::Generic(message) => message.to_owned(),
            Self::InvalidSpecifier => "Section Specifier was not found".into(),
        };
        write!(f, "{}", msg)?;
        Ok(())
    }
}

impl scanner::FromNext for Section {
    type Err = SectionParseError;

    /// parses a Section token out of `s`.
    fn next(s: &str) -> Result<(usize, Self), Self::Err> {
        let specifier_line_error_msg = "Could not parse specifier line";

        let _line_idx = s.find("\n").or_else(|| Some(s.len())).unwrap();

        let mut scan = StrScanner::create(s);
        let specifier_line = scan
            .next_line()
            .or_else(|_e| Err(SectionParseError::Generic(specifier_line_error_msg.into())))?;
        let trimmed_specifier_line = specifier_line.trim();

        // match "\[.*\]" and extract specifier
        if !trimmed_specifier_line.starts_with("[") || !trimmed_specifier_line.ends_with("]") {
            return Err(SectionParseError::Generic(specifier_line_error_msg.into()));
        }
        let _specifier = &specifier_line[1..specifier_line.len() - 1];
        let specifier_tab = StrUtils(&specifier_line).tabs();

        // parse section specifier pattern
        let open_bracket_idx = specifier_line.find("[").unwrap();
        let closed_bracket_idx = specifier_line.find("]").unwrap();
        let specifier = specifier_line[open_bracket_idx + 1..closed_bracket_idx].trim();

        // make sure not to encounter another section, or just parse partially
        // Consume until beginning of new section, end of stream, or end of tab level
        let body_start_cur = scan.cur;
        while let Ok((len, line)) = scan.peek_line() {
            let trimmed_line = line.trim();
            let line_tab = StrUtils(&line).tabs();

            // reached end of tab body
            let line_tab_less_than_section =
                || !line_tab.is_empty() && line_tab.len() < specifier_tab.len();
            let line_has_no_tab_but_section_does =
                || line_tab.is_empty() && !specifier_tab.is_empty() && !trimmed_line.is_empty();
            if line_tab_less_than_section() || line_has_no_tab_but_section_does() {
                break;
            }

            // start of new section detected
            if line_tab.len() == specifier_tab.len()
                && trimmed_line.starts_with("[")
                && trimmed_line.ends_with("]")
            {
                break;
            }
            // consume all other lines
            scan.advance(len);
        }
        let section_body = &scan.stream[body_start_cur..scan.cur];

        Ok((
            scan.cur,
            Section {
                tab: specifier_tab.into(),
                specifier: specifier.into(),
                // body: StrUtils(section_body).untab(specifier_tab)
                body: section_body.into(),
            },
        ))
    }
}

impl std::str::FromStr for Section {
    type Err = SectionParseError;

    /// parses a section from `s`. If `s` contains more than
    /// just the section, discards the rest
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // parse the next Section token from `s`
        // disregard substring
        let (_cur, out) = Self::next(s)?;

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test;

    #[test]
    fn test_section_parsing() {
        assert_parses_as(
            "[Test Section]\n\
            Some Body here!\n\
            And here too!\
            "
            .trim(),
            &Section {
                tab: "".into(),
                specifier: "Test Section".into(),
                body: "Some Body here!\n\
                       And here too!"
                    .into(),
            },
        );

        assert_parses_as(
            r#"    [Habits]
    This is a tabbed entry.
    Do take note of that.
    "#,
            &Section {
                tab: "    ".into(),
                specifier: "Habits".into(),
                body: r#"    This is a tabbed entry.
    Do take note of that.
    "#
                .into(),
            },
        );
    }

    #[test]
    fn test_parse_tabs() {
        assert_parses_as(
            r#"		 [Mixed Tabs Section]
		 Testing that mixed tabs are picked on and expected in the body
"#,
            &Section {
                tab: "		 ".into(),
                specifier: "Mixed Tabs Section".into(),
                body: "		 Testing that mixed tabs are picked on and expected in the body\n".into(),
            },
        );

        assert_parses_as(
            r#"		 [Untabbed Body]
Body not tabbed, body does not belong to section!
"#,
            &Section {
                tab: "		 ".into(),
                specifier: "Untabbed Body".into(),
                body: "".into(),
            },
        );
    }

    #[test]
    fn test_parse_partial_section() {
        assert_parses_as(
            r#"[Section A]
This section describes the recipe for baking a cake.

[Section B]
This section describes the ingredients for the cake

[Section C]
this section is for personal notes."#,
            &Section {
                tab: "".into(),
                specifier: "Section A".into(),
                body: "This section describes the recipe for baking a cake.\n\n".into(),
            },
        );
    }

    #[test]
    fn test_parse_embedded_sections() {
        assert_parses_as(
            r#"[Section A]
    This section describes the recipe for baking a cake.

    [Section A1]
    This section describes the ingredients for the cake

    [Section A2]
    this section is for personal notes."#,
            &Section {
                tab: "".into(),
                specifier: "Section A".into(),
                body: r#"    This section describes the recipe for baking a cake.

    [Section A1]
    This section describes the ingredients for the cake

    [Section A2]
    this section is for personal notes."#
                    .into(),
            },
        );
    }

    fn assert_parses_as(s: &str, exp: &Section) {
        test::assert_parses_as::<Section, SectionParseError>(s, exp);
    }

    fn assert_fails_to_parse_as(s: &str, err: &SectionParseError) {
        let act_err: SectionParseError = s.parse::<Section>().unwrap_err();
        test::assert_variant_eq(&act_err, &err);
    }

    #[test]
    fn test_entry() {}
}
