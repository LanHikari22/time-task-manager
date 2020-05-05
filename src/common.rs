#![allow(dead_code)]
#![allow(unused_macros)]

/// checks if $s is in any occurances in $v
pub fn str_in(s: &str, v: &[&str]) -> bool {
    v.iter().any(|&e| s == e)
}

/// checks if the character $c matches any characters in $v
pub fn char_in(c: char, v: &[char]) -> bool {
    v.iter().any(|&e| c == e)
}

/// checks for occurrance of every character in $chars in $s
/// 
/// # Examples
/// ```
/// assert_eq!(str_contains_any_char("0xAC", "ABCDEFabcdef"), true);
/// assert_eq!(str_contains_any_char("0xAC", "123"), false);
/// ```
pub fn str_contains_any_char(s: &str, chars: &str) -> bool {
    for c in chars.chars() {
        if s.contains(c) {return true;}
    }
    false
}

/// counts the occurance of the character $occurance in string $s
pub fn str_count_char(s: &str, occurance: char) -> i32 {
    s.chars().map(|c| if c == occurance {1} else {0}).sum::<i32>()
}


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
        _ if str_contains_any_char(&s, "ABCDEFabcdef") => result_err_to_unit(i32::from_str_radix(&s, 16).and_then(|i| Ok(neg * i))),
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
