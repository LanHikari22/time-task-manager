use super::stat::*;


#[allow(dead_code)]
#[derive(Debug)]
pub enum WeekDay {
    M, T, W, R, F, S, U,
}

#[derive(Debug, PartialEq)]
pub struct BlockTrackerEntry {
    entry_name: String,
    week_stats: [Stat; 7],
}

impl std::ops::Index<WeekDay> for BlockTrackerEntry {
    type Output = Stat;
    fn index(&self, idx: WeekDay) -> &Stat {
        &self.week_stats[idx as usize]
    }
}

#[derive(Debug)]
pub enum BlockTrackerEntryParseError {
    StatParseError (usize),
    TooManyEntryTokens,
    TooFewEntryTokens,
}

impl std::str::FromStr for BlockTrackerEntry {
    type Err = BlockTrackerEntryParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // convert spacing to one spacing
        let s = s.split_whitespace().collect::<Vec<&str>>().join(" ");
        let mut entry_name: String = "".to_string();
        let mut week_stats: [Stat; 7] = [Stat::Unknown; 7];
        let mut entry_count = 0;
        for (i, token) in s.split(' ').enumerate() {
            println!("token {} {}", i, token);
            // invalid block tracker entry, they are only a stat per day and then the name.
            if i > 7 {
                return Err(BlockTrackerEntryParseError::TooManyEntryTokens);
            }

            if i == 7 {
                entry_name = token.to_string();
            }
            else {
                // parse day stat
                let res = token.parse::<Stat>();
                if res.is_err() {
                    return Err(BlockTrackerEntryParseError::StatParseError(i));
                }
                week_stats[i] = res.unwrap();
            }

            entry_count += 1;
        }

        // ensure we traversed all expected tokens
        if entry_count != 8 {
            return Err(BlockTrackerEntryParseError::TooFewEntryTokens);
        }

        println!("week_stats {:?}", week_stats);

        Ok(BlockTrackerEntry {entry_name, week_stats})
    }
}

#[cfg(test)]
mod tests {
    
    #[allow(unused_imports)] use super::*;
}
    // ------------------------------------------------------------------------------------------------------------------
    // BlockTracker Tests -----------------------------------------------------------------------------------------------
    // ------------------------------------------------------------------------------------------------------------------

    #[test]
    fn test_block_tracker_entry_parse() {
        fn assert_parses(s: &str, exp: BlockTrackerEntry) {
            println!("test {}", s);
            s.parse::<BlockTrackerEntry>()
                .and_then(|res| {assert_eq!(res, exp); Ok(res)})
                .expect("failed to parse:");
        }


        assert_parses("0  0  0  0  0  0  0   PROJECT", BlockTrackerEntry {entry_name: "PROJECT".to_string(), week_stats: [Stat::Count{act: Some(0), exp: None};7]});
        assert_parses("?  ?  ?  ?  ?  ?  ?   PROJECT", BlockTrackerEntry {entry_name: "PROJECT".to_string(), week_stats: [Stat::Unknown;7]});
        assert_parses("-  -  -  -  -  -  -   PR0JECT", BlockTrackerEntry {entry_name: "PR0JECT".to_string(), week_stats: [Stat::Bool {act: false, exp: true};7]});

        let bool_stat = Stat::Bool {act: true, exp: true};
        assert_parses("?  !  !  !  !  /- 4/4 PR0JECT", BlockTrackerEntry {entry_name: "PR0JECT".to_string(), 
            week_stats: [Stat::Unknown, bool_stat, bool_stat, bool_stat, bool_stat, 
                         Stat::Bool {act: false, exp: false}, Stat::Count {act: Some(4), exp: Some(4)}]});
    }
