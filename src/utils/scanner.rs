//! @author LanHikari22 <lanhikarixx@gmail.com>
//! 
//! This defines the FromNext trait and StrScanner type which can be used to parse elements off of a
//! string stream and maintain a running substr window such that each element can be consumed consecutively
//! to define more complex combinator parsing logic. It also defines scan<Type> methods for common elements

#![allow(dead_code)]
use super::common::CharUtils;


pub struct StrScanner<'a> {
    // defines the string stream to consume from 
    pub stream: &'a str,
    // cursor into the stream used to obtain the current unconsumed slice
    pub cur: usize,
}

impl<'a> StrScanner<'a> {
    pub fn peek<T: FromNext>(&mut self) -> Result<(usize, T), T::Err> {
        let (cur, elem) = T::next(&self.stream[self.cur..])?;
        assert!(cur >= self.cur);
        Ok((cur, elem))
    }

    pub fn advance(&mut self, n: usize) {
        assert!(self.cur + n <= self.stream.len());
        self.cur += n;
    }

    pub fn rewind(&mut self, n: usize) {
        self.cur -= n;
    }

    pub fn next<T: FromNext>(&mut self) -> Result<T, T::Err> {
        let (len, elem) = self.peek::<T>()?;
        self.advance(len);
        Ok(elem)
    }

    /// gets the next token and separator but does not advance the cursor
    pub fn peek_token(&mut self, end: fn(&str) -> Option<usize>) -> Result<(usize, String, String), ()> {
        if self.stream[self.cur..].is_empty() {return Err(())}
        for (i, _) in (&self.stream[self.cur..]).char_indices() {
            let token_len = end(&self.stream[self.cur+i..]);
            if token_len.is_some() {
                let token = &self.stream[self.cur..self.cur + i];
                let sep = &self.stream[self.cur + i..self.cur + i + token_len.unwrap()];
                return Ok((i+token_len.unwrap(), token.into(), sep.into()))
            }
        }
        Ok((self.stream.len() - self.cur, self.stream[self.cur..].into(), "".into()))
    }

    /// Scans for the next token given a specified separator and retrieves both the token and separator
    /// ### Examples
    /// ```
    /// let mut scanner = StrScanner::create("Comma, Separated, Values!, 999");
    /// let is_sep = |s: &str| if s.starts_with(',') {Some(1)} else {None};
    /// assert_eq!(scanner.next_token(is_sep), Ok(("Comma".into(),      ",".into())));
    /// assert_eq!(scanner.next_token(is_sep), Ok((" Separated".into(), ",".into())));
    /// assert_eq!(scanner.next_token(is_sep), Ok((" Values!".into(),   ",".into())));
    /// assert_eq!(&scanner.stream[scanner.cur..], " 999");
    /// assert_eq!(scanner.next_token(is_sep), Ok((" 999".into(),       "".into())));
    /// assert_eq!(scanner.next_token(is_sep), Err(()));
    /// ```
    ///
    /// ### Parameters
    /// - `end`: A callback that takes the stream's current position and determines if we're at the stopping separator
    ///          and its length
    pub fn next_token(&mut self, end: fn(&str) -> Option<usize>) -> Result<(String, String), ()> {
        let (len, token, sep) = self.peek_token(end)?;
        self.advance(len);
        Ok((token, sep))
    }

    pub fn peek_word(&mut self) -> Result<(usize, String), ()> {
        if self.stream[self.cur..].trim().is_empty() {return Err(())}
        let mut trim_state = true;
        for (i, c) in (&self.stream[self.cur..]).char_indices() {
            if !trim_state && c.is_whitespace() {
                let out = self.stream[self.cur..self.cur + i].trim();
                return Ok((i+1, out.into()))
            }
            if trim_state && !c.is_whitespace() {trim_state = false;}
        }

        Ok((self.stream.len() - self.cur, self.stream[self.cur..].into()))
    }

    /// # Examples
    /// ```
    /// let mut scanner = StrScanner::create("I love cereal!");
    /// assert_eq!(scanner.next_word(), Ok("I".into()));
    /// assert_eq!(scanner.next_word(), Ok("love".into()));
    /// assert_eq!(scanner.next_word(), Ok("cereal!".into()));
    /// assert_eq!(scanner.next_word(), Err(()));

    /// let mut scanner = StrScanner::create("   Trim   your spaces!   ");
    /// assert_eq!(scanner.next_word(), Ok("Trim".into()));
    /// assert_eq!(scanner.next_word(), Ok("your".into()));
    /// assert_eq!(scanner.next_word(), Ok("spaces!".into()));
    /// assert_eq!(scanner.next_word(), Err(()));
    /// ```
    pub fn next_word(&mut self) -> Result<String, ()> {
        let (len, out) = self.peek_word()?;
        self.advance(len);
        Ok(out)
    }

    /// reads the next line from the stream but does not advance the cursor
    pub fn peek_line(&mut self) -> Result<(usize, String), ()> {
        let (cur, out, _) = self.peek_token(
            |s| if s.starts_with("\n") {Some(1)} else {None})?;
        Ok((cur, out.trim_end().into()))
    }

    pub fn next_line(&mut self) -> Result<String, ()> {
        let (cur, out) = self.peek_line()?;
        self.advance(cur);
        Ok(out)
    }

    pub fn peek_char(&mut self) -> Result<(usize, char), ()> {
        println!("len: {}, cur: {}", self.stream.len(), self.cur);
        if self.cur == self.stream.len() {return Err(());}
        let out: char = self.stream[self.cur..].chars().next().unwrap();
        Ok((out.len_utf8(), out))
    }

    pub fn next_char(&mut self) -> Result<char, ()> {
        let (len, out) = self.peek_char()?;
        self.advance(len);
        Ok(out)
    }

    /// scans the next int but does not advance the stream cursor
    pub fn peek_int(&mut self) -> Result<(usize, i32), ()> {
        let sign: i32 = if self.match_next("-").is_ok() {-1} else {1};
        let mut total_step = if sign == -1 {1} else {0};

        let mut num: i32 = 0;
        let mut is_valid = false;
        loop {
            let tup_res = self.peek_char();
            if tup_res.is_err() {break;}
            let (step, c) = tup_res.unwrap();

            if !CharUtils(c).is_in("0-9") {break;}
            is_valid = true;
            num = 10*num + c.to_digit(10).unwrap() as i32;
            self.advance(step);
            total_step += step;
        }

        if !is_valid {return Err(())}
        
        self.rewind(total_step);
        Ok((total_step, sign * num))
    }

    /// an integer may only numbers 0-9, and an optional negative sign: <-?><[0-9][0-9]+>
    /// this does not require that the integer is separated by white space or anything
    ///
    /// # Examples
    /// ```
    /// assert_eq!(StrScanner::create("10").next_int(), Ok(10));
    /// assert_eq!(StrScanner::create("00").next_int(), Ok(0));
    /// assert_eq!(StrScanner::create("-999").next_int(), Ok(-999));
    /// assert_eq!(StrScanner::create("not a valid int!").next_int(), Err(()));
    /// assert_eq!(StrScanner::create("100pancakes!").next_int(), Ok(100));
    /// ```
    pub fn next_int(&mut self) -> Result<i32, ()> {
        let (step, out) = self.peek_int()?;
        self.advance(step);
        Ok(out)
    }

    /// matches the next characters to `exp`
    pub fn match_next(&mut self, exp: &str) -> Result<(), ()> {
        if (&self.stream[self.cur..]).starts_with(exp) {
            self.cur += exp.len();
            return Ok(())
        }
        Err(())
    }

    pub fn create(stream: &'a str) -> Self {
        Self {stream: stream, cur: 0}
    }
}

/// This trait allows for any type to be stream parsed with `StrScanner().next<T>()`
pub trait FromNext: Sized {
    type Err;

    /// stream parse one token of type `Self` from `s` and specify how much it advanced the stream cursor
    fn next(s: &str) -> Result<(usize, Self), Self::Err>;
}



#[cfg(test)]
mod tests {
    use super::*;

    mod str_scanner_tests {
        use super::*;

        #[test]
        fn test_next_int() {
            assert_eq!(StrScanner::create("10").next_int(), Ok(10));
            assert_eq!(StrScanner::create("00").next_int(), Ok(0));
            assert_eq!(StrScanner::create("-999").next_int(), Ok(-999));
            assert_eq!(StrScanner::create("not a valid int!").next_int(), Err(()));
            assert_eq!(StrScanner::create("100pancakes!").next_int(), Ok(100));

            let mut scanner = StrScanner::create("192.168.1.1");
            assert_eq!(scanner.next_int(), Ok(192));
            assert_eq!(scanner.next_char(), Ok('.'));
            assert_eq!(scanner.next_int(), Ok(168));
            assert_eq!(scanner.next_char(), Ok('.'));
            assert_eq!(scanner.next_int(), Ok(1));
            assert_eq!(scanner.next_char(), Ok('.'));
            assert_eq!(scanner.next_int(), Ok(1));
            assert_eq!(scanner.next_int(), Err(()));
        }

        #[test]
        fn test_next_word() {
            let mut scanner = StrScanner::create("I love cereal!");
            assert_eq!(scanner.next_word(), Ok("I".into()));
            assert_eq!(scanner.next_word(), Ok("love".into()));
            assert_eq!(scanner.next_word(), Ok("cereal!".into()));
            assert_eq!(scanner.next_word(), Err(()));

            let mut scanner = StrScanner::create("   Trim   your spaces!   ");
            assert_eq!(scanner.next_word(), Ok("Trim".into()));
            assert_eq!(scanner.next_word(), Ok("your".into()));
            assert_eq!(scanner.next_word(), Ok("spaces!".into()));
            assert_eq!(scanner.next_word(), Err(()));
        }


        #[test]
        fn test_next_token() {
            let mut scanner = StrScanner::create("Comma, Separated, Values!, 999");
            let is_sep = |s: &str| if s.starts_with(',') {Some(1)} else {None};
            assert_eq!(scanner.next_token(is_sep), Ok(("Comma".into(),      ",".into())));
            assert_eq!(scanner.next_token(is_sep), Ok((" Separated".into(), ",".into())));
            assert_eq!(scanner.next_token(is_sep), Ok((" Values!".into(),   ",".into())));
            assert_eq!(&scanner.stream[scanner.cur..], " 999");
            assert_eq!(scanner.next_token(is_sep), Ok((" 999".into(),       "".into())));
            assert_eq!(scanner.next_token(is_sep), Err(()));
        }

        #[test]
        fn test_next_line() {
            let mut scanner = StrScanner::create("
            Roses are red.\r
            Violets are blue!
            I thought...    
            I could be with you!");

            assert_eq!(scanner.next_line(), Ok("".into()));
            assert_eq!(scanner.next_line(), Ok("            Roses are red.".into()));
            assert_eq!(scanner.next_line(), Ok("            Violets are blue!".into()));
            assert_eq!(scanner.next_line(), Ok("            I thought...".into()));
            assert_eq!(scanner.next_line(), Ok("            I could be with you!".into()));
            assert_eq!(scanner.next_line(), Err(()));
        }
    }
}