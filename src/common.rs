/// checks if $s is in any occurances in $v
pub fn str_in(s: &str, v: &[&str]) -> bool {
    v.iter().any(|&e| s == e)
}

/// checks if the character $c matches any characters in $v
pub fn char_in(c: char, v: &[char]) -> bool {
    v.iter().any(|&e| c == e)
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
        _ => result_err_to_unit(s.parse::<i32>().and_then(|i| Ok(neg * i))),
    }
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
}
