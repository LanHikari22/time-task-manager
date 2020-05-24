#![allow(dead_code)]
#![allow(unused_macros)]

// -----------------------------------------------------------------
// -----------------------------------------------------------------
/// implements search and scanning functions for &str
pub struct StrUtils<'a>(pub &'a str);

impl StrUtils<'_> {
    /// checks for occurrance of every character in $chars in the string
    /// 
    /// # Examples
    /// ```
    /// assert_eq!(StrUtils("0xDeadFeed").contains_any("ABCDEFabcdef"), true);
    /// assert_eq!(StrUtils("456").contains_any("123"), false);
    /// ```
    pub fn contains_any(&self, chars: &str) -> bool {
        for c in chars.chars() {
            if self.0.contains(c) {return true;}
        }
        false
    }

    /// checks that every character specified in $chars is contained in the string
    /// 
    /// # Examples
    /// ```
    /// assert_eq!(StrUtils("[ContainsBrackets]").contains_all("[]"), true);
    /// assert_eq!(StrUtils("[01]").contains_all("01"), true);
    /// assert_eq!(StrUtils("[1]").contains_all("01"), false);
    /// ```
    pub fn contains_all(&self, chars: &str) -> bool {
        for c in chars.chars() {
            if !self.0.contains(c) {return false;}
        }
        true
    }

    /// checks if the string is in any occurances in $v
    /// 
    /// # Examples
    /// ```
    /// assert_eq!(StrUtils("OK").in_any(&vec!["OK", "PASS"]), true);
    /// assert_eq!(StrUtils("NO").in_any(&vec!["OK", "PASS"]), false);
    /// ```
    pub fn in_any(&self, v: &[&str]) -> bool {
        v.iter().any(|&el| self.0 == el)
    }

    /// counts the occurance of the character $occurance in the string
    pub fn count_char(&self, occurance: char) -> i32 {
        self.0.chars().map(|c| if c == occurance {1} else {0}).sum::<i32>()
    }

    /// gets the tab characters of the string
    pub fn tabs(&self) -> &str {
        let mut idx = 0;
        for c in self.0.chars() {
            if !CharUtils(c).is_in(&vec![' ', '\t']) {break;}
            idx += 1;
        }
        &self.0[..idx]
    }

}

impl<'a> From<&'a str> for StrUtils<'a> {
    fn from(s: &'a str) -> Self {
        StrUtils(s)
    }
}


#[cfg(test)]
mod string_utils_tests {
    use super::*;
    #[test]
    fn test_contains_any() {
        assert_eq!(StrUtils("0xDeadFeed").contains_any("ABCDEFabcdef"), true);
        assert_eq!(StrUtils("456").contains_any("123"), false);
    }

    #[test]
    fn test_contains_all() {
        assert_eq!(StrUtils("[ContainsBrackets]").contains_all("[]"), true);
        assert_eq!(StrUtils("[01]").contains_all("01"), true);
        assert_eq!(StrUtils("[1]").contains_all("01"), false);
    }

    fn test_in_any() {
        assert_eq!(StrUtils("OK").in_any(&vec!["OK", "PASS"]), true);
        assert_eq!(StrUtils("NO").in_any(&vec!["OK", "PASS"]), false);
    }


}


/// -----------------------------------------------------------------
/// -----------------------------------------------------------------

/// utils for the char type
pub struct CharUtils(char);

impl CharUtils {
    /// checks if the character matches any characters in $v
    pub fn is_in(&self, v: &[char]) -> bool {
        v.iter().any(|&el| self.0 == el)
    }
}

/// -----------------------------------------------------------------
/// -----------------------------------------------------------------

/// this can be used to handle multiple result sources without subtyping them together
pub fn result_err_to_unit<T, E>(res: Result<T, E>) -> Result<T, ()> {
    match res {
        Ok(res) => return Ok(res),
        Err(_) => Err(()),
    }
}

/// parses decimal, hexadecimal, octal and binary integer strings
pub fn parse_integer_auto(s: &str) -> Result<i32, ()> {
    let neg = if s.starts_with("-") {-1} else {1};
    let s = s.replace("-", "");

    match s {
        _ if s.starts_with("0x") => result_err_to_unit(i32::from_str_radix(&s[2..], 16).and_then(|i| Ok(neg * i))),
        _ if s.starts_with("0o") => result_err_to_unit(i32::from_str_radix(&s[2..], 8).and_then(|i| Ok(neg * i))),
        _ if s.starts_with("0b") => result_err_to_unit(i32::from_str_radix(&s[2..], 2).and_then(|i| Ok(neg * i))),
        _ if StrUtils(&s).contains_any("ABCDEFabcdef") => result_err_to_unit(i32::from_str_radix(&s, 16).and_then(|i| Ok(neg * i))),
        _ => result_err_to_unit(s.parse::<i32>().and_then(|i| Ok(neg * i))),
    }
}


/* // just really not giving up on hacking macros to evaluate counters i guess!
macro_rules! num_encode32 {
    {a} => {0};
    {a a} => {1};
    {a a a} => {2};
    {a a a a} => {3};
    {a a a a a} => {4};
    {a a a a a a} => {5};
    {a a a a a a a} => {6};
}

macro_rules! num_decode32 {
    {0} => {a};
    {1} => {a a};
    {2} => {a a a};
    {3} => {a a a a};
    {4} => {a a a a a};
    {5} => {a a a a a a};
    {6} => {a a a a a a a};
}

macro_rules! counter32 {
    {$($decoded: ident)*, +} => {counter32!($($decoded)*), ++};
    {$($decoded: ident)* $next: ident, ++} => {$($decoded)* $next $next};
    {$($decoded: ident)*, -} => {counter32!($($decoded)*, --)};
    {$($decoded: ident)* $next: ident, --} => {$($decoded)*};
}

macro_rules! hello_repeater {
    {$n:expr} => {println!("{}. hiii!", $n); hello_repeater!(counter32!($n-1));};
    {0} => {};
}

macro_rules! test_macro {
    {0} => {
        hello_repeater!(3);
    };
}
*/

/// this takes the painful repetitive nature of trying to repeat actions from macros and allows a
/// macro to be repeated 0..32 times.
macro_rules! repeat_varargs_macro32 {
    (31, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(30, $macro, $($varargs)*);};
    (30, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(29, $macro, $($varargs)*);};
    (29, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(28, $macro, $($varargs)*);};
    (28, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(27, $macro, $($varargs)*);};
    (27, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(26, $macro, $($varargs)*);};
    (26, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(25, $macro, $($varargs)*);};
    (25, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(24, $macro, $($varargs)*);};
    (24, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(23, $macro, $($varargs)*);};
    (23, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(22, $macro, $($varargs)*);};
    (22, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(21, $macro, $($varargs)*);};
    (21, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(20, $macro, $($varargs)*);};
    (20, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(19, $macro, $($varargs)*);};
    (19, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(18, $macro, $($varargs)*);};
    (18, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(17, $macro, $($varargs)*);};
    (17, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(16, $macro, $($varargs)*);};
    (16, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(15, $macro, $($varargs)*);};
    (15, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(14, $macro, $($varargs)*);};
    (14, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(13, $macro, $($varargs)*);};
    (13, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(12, $macro, $($varargs)*);};
    (12, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(11, $macro, $($varargs)*);};
    (11, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(10, $macro, $($varargs)*);};
    (10, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(9, $macro, $($varargs)*);};
    (9, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(8, $macro, $($varargs)*);};
    (8, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(7, $macro, $($varargs)*);};
    (7, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(6, $macro, $($varargs)*);};
    (6, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(5, $macro, $($varargs)*);};
    (5, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(4, $macro, $($varargs)*);};
    (4, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(3, $macro, $($varargs)*);};
    (3, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(2, $macro, $($varargs)*);};
    (2, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(1, $macro, $($varargs)*);};
    (1, $macro:ident, $($varargs: expr),*) => {$macro!($($varargs)*); repeat_varargs_macro32!(0, $macro, $($varargs)*);};
    (0, $macro:ident, $($varargs: expr),*) => {};
}

/// this takes the painful repetitive nature of trying to repeat actions from macros and allows a
/// macro to be repeated 0..32 times. The macros are expected to be index-aware, as they take it as
/// first argument.
macro_rules! repeat_enumerate_varargs_macro32 {
    (31, $macro:ident, $($varargs: expr),*) => {$macro!(30, $($varargs)*); repeat_enumerate_varargs_macro32!(30, $macro, $($varargs)*);};
    (30, $macro:ident, $($varargs: expr),*) => {$macro!(29, $($varargs)*); repeat_enumerate_varargs_macro32!(29, $macro, $($varargs)*);};
    (29, $macro:ident, $($varargs: expr),*) => {$macro!(28, $($varargs)*); repeat_enumerate_varargs_macro32!(28, $macro, $($varargs)*);};
    (28, $macro:ident, $($varargs: expr),*) => {$macro!(27, $($varargs)*); repeat_enumerate_varargs_macro32!(27, $macro, $($varargs)*);};
    (27, $macro:ident, $($varargs: expr),*) => {$macro!(26, $($varargs)*); repeat_enumerate_varargs_macro32!(26, $macro, $($varargs)*);};
    (26, $macro:ident, $($varargs: expr),*) => {$macro!(25, $($varargs)*); repeat_enumerate_varargs_macro32!(25, $macro, $($varargs)*);};
    (25, $macro:ident, $($varargs: expr),*) => {$macro!(24, $($varargs)*); repeat_enumerate_varargs_macro32!(24, $macro, $($varargs)*);};
    (24, $macro:ident, $($varargs: expr),*) => {$macro!(23, $($varargs)*); repeat_enumerate_varargs_macro32!(23, $macro, $($varargs)*);};
    (23, $macro:ident, $($varargs: expr),*) => {$macro!(22, $($varargs)*); repeat_enumerate_varargs_macro32!(22, $macro, $($varargs)*);};
    (22, $macro:ident, $($varargs: expr),*) => {$macro!(21, $($varargs)*); repeat_enumerate_varargs_macro32!(21, $macro, $($varargs)*);};
    (21, $macro:ident, $($varargs: expr),*) => {$macro!(20, $($varargs)*); repeat_enumerate_varargs_macro32!(20, $macro, $($varargs)*);};
    (20, $macro:ident, $($varargs: expr),*) => {$macro!(19, $($varargs)*); repeat_enumerate_varargs_macro32!(19, $macro, $($varargs)*);};
    (19, $macro:ident, $($varargs: expr),*) => {$macro!(18, $($varargs)*); repeat_enumerate_varargs_macro32!(18, $macro, $($varargs)*);};
    (18, $macro:ident, $($varargs: expr),*) => {$macro!(17, $($varargs)*); repeat_enumerate_varargs_macro32!(17, $macro, $($varargs)*);};
    (17, $macro:ident, $($varargs: expr),*) => {$macro!(16, $($varargs)*); repeat_enumerate_varargs_macro32!(16, $macro, $($varargs)*);};
    (16, $macro:ident, $($varargs: expr),*) => {$macro!(15, $($varargs)*); repeat_enumerate_varargs_macro32!(15, $macro, $($varargs)*);};
    (15, $macro:ident, $($varargs: expr),*) => {$macro!(14, $($varargs)*); repeat_enumerate_varargs_macro32!(14, $macro, $($varargs)*);};
    (14, $macro:ident, $($varargs: expr),*) => {$macro!(13, $($varargs)*); repeat_enumerate_varargs_macro32!(13, $macro, $($varargs)*);};
    (13, $macro:ident, $($varargs: expr),*) => {$macro!(12, $($varargs)*); repeat_enumerate_varargs_macro32!(12, $macro, $($varargs)*);};
    (12, $macro:ident, $($varargs: expr),*) => {$macro!(11, $($varargs)*); repeat_enumerate_varargs_macro32!(11, $macro, $($varargs)*);};
    (11, $macro:ident, $($varargs: expr),*) => {$macro!(10, $($varargs)*); repeat_enumerate_varargs_macro32!(10, $macro, $($varargs)*);};
    (10, $macro:ident, $($varargs: expr),*) => {$macro!(9, $($varargs)*); repeat_enumerate_varargs_macro32!(9, $macro, $($varargs)*);};
    (9, $macro:ident, $($varargs: expr),*) => {$macro!(8, $($varargs)*); repeat_enumerate_varargs_macro32!(8, $macro, $($varargs)*);};
    (8, $macro:ident, $($varargs: expr),*) => {$macro!(7, $($varargs)*); repeat_enumerate_varargs_macro32!(7, $macro, $($varargs)*);};
    (7, $macro:ident, $($varargs: expr),*) => {$macro!(6, $($varargs)*); repeat_enumerate_varargs_macro32!(6, $macro, $($varargs)*);};
    (6, $macro:ident, $($varargs: expr),*) => {$macro!(5, $($varargs)*); repeat_enumerate_varargs_macro32!(5, $macro, $($varargs)*);};
    (5, $macro:ident, $($varargs: expr),*) => {$macro!(4, $($varargs)*); repeat_enumerate_varargs_macro32!(4, $macro, $($varargs)*);};
    (4, $macro:ident, $($varargs: expr),*) => {$macro!(3, $($varargs)*); repeat_enumerate_varargs_macro32!(3, $macro, $($varargs)*);};
    (3, $macro:ident, $($varargs: expr),*) => {$macro!(2, $($varargs)*); repeat_enumerate_varargs_macro32!(2, $macro, $($varargs)*);};
    (2, $macro:ident, $($varargs: expr),*) => {$macro!(1, $($varargs)*); repeat_enumerate_varargs_macro32!(1, $macro, $($varargs)*);};
    (1, $macro:ident, $($varargs: expr),*) => {$macro!(0, $($varargs)*); repeat_enumerate_varargs_macro32!(0, $macro, $($varargs)*);};
    (0, $macro:ident, $($varargs: expr),*) => {};
}

macro_rules! rept_list {
    ($($e: expr; $n32: expr),*) => {[$(rept_list!($e; $n32))*]};
    ($e: expr; $n32: expr) => {repeat_varargs_macro32!($n32, rept_list, $e)};
    ($e: expr) => {$e};
}

fn bad_warning_macros() {
    repeat_varargs_macro32!(2, println, "Wowies!");
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_integer_auto() {
        fn run_test(s: &str, exp: i32) {
            println!("test: {}", s);
            assert_eq!(parse_integer_auto(s).unwrap(), exp);
        }
        
        run_test("10", 10);
        run_test("0x10", 0x10);
        run_test("-0x10", -0x10);
        run_test("-0b01010101", -0b01010101);
        run_test("0o777", 0o777);
    }

    #[test]
    fn test_macros() {
        // repeat_varargs_macro32!(20, println, "Wowies!");
        // println!("rept_list: {}", rept_list!(0; 3, 1; 2));
        // panic!("whaaaaat");
    }
}
