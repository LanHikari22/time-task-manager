//! @author LanHikari22 <lanhikarixx@gmail.com>
//! 
//! This defines the FromNext trait and StrScanner type which can be used to parse elements off of a
//! string stream and maintain a running substr window such that each element can be consumed consecutively
//! to define more complex combinator parsing logic. It also defines scan<Type> methods for common elements

struct StrScanner<'a> {
    // defines the string stream to consume from 
    stream: &'a str,
    // cursor into the stream used to obtain the current unconsumed slice
    cur: usize,
}

impl<'a> StrScanner<'a> {
    pub fn scan<T: FromNext>(&mut self) -> Result<T, T::Err> {
        let (elem, cur) = T::next(self.stream)?;
        assert!(cur >= self.cur);
        self.cur = self.cur;
        Ok(elem)
    }


    /// scans for simple regex-like string patterns
    /// ## Supported Syntax:
    ///     - characters are expected to match as-is.
    ///     - anything inside [] can be matched. Use - for range like a-zA-Z, 0-9. 
    ///     - wild cards * (>=0), ? (0 or 1), and + (>=1) can be utilized.
    ///     - () can be used for operation grouping
    ///     - <> is a capture group. No nesting is allowed. wild cards applied to this grouping extend its capture like <...>*
    ///       only outermost captures are considered. inner captures <<>> are ignored for composability: <({int})>
    /// ## Features:
    ///     X No back-stepping, maximum greediness matching expected. (Ex: a*a will NOT match, as a* consumes all a's)
    pub fn scan_pat(&mut self, pat: &str) -> Result<Vec<String>, String> {
        let mut captures: Vec<(usize, usize)> = vec![];
        let mut capturing: bool = false;
        let mut open_paren_stack: Vec<usize> = vec![];

        let mut pat_cur = 0;
        for (pat_unit, size) in Self::next_pat(&pat[pat_cur..]){

            // advance to next pattern unit
            pat_cur += size;
            
        }

        unimplemented!();
    }

    /// an integer may only numbers 0-9, and an optional negative sign: <-?><[0-9][0-9]+>
    pub fn scan_int(&mut self) -> Result<i32, String> {
        unimplemented!();
    }

    pub fn create(stream: &'a str) -> Self {
        Self {stream: stream, cur: 0}
    }

    enum Wildcard {
        ZeroOrOne, ZeroOrMany, OneOrMany,
    }

    impl WildCard {
        fn from(c: char) -> Result<Self, ()> {
            match c {
                '?' => Ok(Self::ZeroOrOne),
                '*' => Ok(Self::ZeroOrMany),
                '+' => Ok(Self::OneOrMany),
                _ => (Err(()))
            }
        }
    }

    enum NextPat<'a> {
        Match(&'a str),
        Range()
        Group(Vec<NextPat<'a>>),
        Wild(Wildcard),
        Capture(bool),
    }

    fn next_pat(pat: &'a str) -> (NextPat, usize) {
        unimplemented!();
    }
}

trait FromNext: Sized {
    type Err;
    fn next(s: &str) -> Result<(Self, usize), Self::Err>;
}



#[cfg(test)]
mod tests {
    use super::*;

    mod str_scanner_tests {
        use super::*;
        #[test]
        fn test_scan_pat() {

        }

        fn assert_matches_scan_pat(s: &str, pat: &str) {
            let mut scan = StrScanner::create(s);
            let res = scan.scan_pat(pat);
            assert!(res.is_ok());


        }
    }

    #[test]
    fn test() {
        println!("hi!");
        assert_eq!(1,2);
    }

}