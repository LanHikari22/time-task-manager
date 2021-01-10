/// checks that two enum variants are equal, independent of their value
pub fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

/// asserts that two enum variants are equal, independent of their value
pub fn assert_variant_eq<T>(a: &T, b: &T) {
    assert_eq!(std::mem::discriminant(a), std::mem::discriminant(b));
}

pub fn assert_parses_as<T,E>(s: &str, exp: &T)
where
    T: std::str::FromStr + std::fmt::Debug + std::cmp::PartialEq,
    E: std::convert::From<<T as std::str::FromStr>::Err> + std::fmt::Display,
{
    let res = s.parse::<T>();
    match res {
        Ok(act) => assert_eq!(&act, exp),
        Err(e) => panic!(format!(
            "\nExpected \"{}\" to parse but got {}: \"{}\"\n",
            s, std::any::type_name::<E>(), E::from(e))),
    }
}
