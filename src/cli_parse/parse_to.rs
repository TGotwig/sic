use crate::cli_parse::core::numbers::F32;

#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub enum ParsePerTypeError {
    NoSuchElement,
    ParseIntError,
    ParseUIntError,
    ParseFloatError,
    InvalidModifierSyntax,
    InvalidOperationForModifier,
    ModifierArgumentExpected,
}

/// Based on sic_parser::value_parser
pub trait ParseFromIter {
    type Error;

    fn parse<'a, I>(iterator: &mut I) -> Result<Self, Self::Error>
    where
        I: Iterator,
        I::Item: AsRef<str> + std::fmt::Debug,
        Self: std::marker::Sized;
}

macro_rules! parse_item {
    ($iterator:expr, $ty:ty, $err:expr) => {{
        let item = $iterator.next().ok_or(ParsePerTypeError::NoSuchElement);
        let item = item.and_then(|v| v.as_ref().parse::<$ty>().map_err(|_| $err))?;
        item
    }};
}

impl ParseFromIter for f32 {
    type Error = ParsePerTypeError;

    fn parse<'a, I>(iterator: &mut I) -> Result<Self, Self::Error>
    where
        I: Iterator,
        I::Item: AsRef<str> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let next1 = parse_item!(iterator, f32, ParsePerTypeError::ParseFloatError);
        Ok(next1)
    }
}

impl ParseFromIter for F32 {
    type Error = ParsePerTypeError;

    fn parse<'a, I>(iterator: &mut I) -> Result<Self, Self::Error>
    where
        I: Iterator,
        I::Item: AsRef<str> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let result: Result<f32, Self::Error> = ParseFromIter::parse(iterator);
        result.map(|v| F32(v))
    }
}

impl ParseFromIter for i32 {
    type Error = ParsePerTypeError;

    fn parse<'a, I>(iterator: &mut I) -> Result<Self, Self::Error>
    where
        I: Iterator,
        I::Item: AsRef<str> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let next1 = parse_item!(iterator, i32, ParsePerTypeError::ParseIntError);
        Ok(next1)
    }
}

impl ParseFromIter for (u32, u32) {
    type Error = ParsePerTypeError;

    fn parse<'a, I>(iterator: &mut I) -> Result<Self, Self::Error>
    where
        I: Iterator,
        I::Item: AsRef<str> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let next1 = parse_item!(iterator, u32, ParsePerTypeError::ParseUIntError);
        let next2 = parse_item!(iterator, u32, ParsePerTypeError::ParseUIntError);

        Ok((next1, next2))
    }
}

impl ParseFromIter for (F32, i32) {
    type Error = ParsePerTypeError;

    fn parse<'a, I>(iterator: &mut I) -> Result<Self, Self::Error>
    where
        I: Iterator,
        I::Item: AsRef<str> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let next1 = parse_item!(iterator, f32, ParsePerTypeError::ParseFloatError);
        let next2 = parse_item!(iterator, i32, ParsePerTypeError::ParseIntError);

        Ok((F32(next1), next2))
    }
}

impl ParseFromIter for (u32, u32, u32, u32) {
    type Error = ParsePerTypeError;

    fn parse<'a, I>(iterator: &mut I) -> Result<Self, Self::Error>
    where
        I: Iterator,
        I::Item: AsRef<str> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let next1 = parse_item!(iterator, u32, ParsePerTypeError::ParseUIntError);
        let next2 = parse_item!(iterator, u32, ParsePerTypeError::ParseUIntError);
        let next3 = parse_item!(iterator, u32, ParsePerTypeError::ParseUIntError);
        let next4 = parse_item!(iterator, u32, ParsePerTypeError::ParseUIntError);

        Ok((next1, next2, next3, next4))
    }
}

impl ParseFromIter for [F32; 9] {
    type Error = ParsePerTypeError;

    fn parse<'a, I>(iterator: &mut I) -> Result<Self, Self::Error>
    where
        I: Iterator,
        I::Item: AsRef<str> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let next1: Result<F32, Self::Error> = ParseFromIter::parse(iterator);
        let next1 = next1?;
        let next2: Result<F32, Self::Error> = ParseFromIter::parse(iterator);
        let next2 = next2?;
        let next3: Result<F32, Self::Error> = ParseFromIter::parse(iterator);
        let next3 = next3?;
        let next4: Result<F32, Self::Error> = ParseFromIter::parse(iterator);
        let next4 = next4?;
        let next5: Result<F32, Self::Error> = ParseFromIter::parse(iterator);
        let next5 = next5?;
        let next6: Result<F32, Self::Error> = ParseFromIter::parse(iterator);
        let next6 = next6?;
        let next7: Result<F32, Self::Error> = ParseFromIter::parse(iterator);
        let next7 = next7?;
        let next8: Result<F32, Self::Error> = ParseFromIter::parse(iterator);
        let next8 = next8?;
        let next9: Result<F32, Self::Error> = ParseFromIter::parse(iterator);
        let next9 = next9?;

        Ok([
            next1, next2, next3, next4, next5, next6, next7, next8, next9,
        ])
    }
}

#[cfg(test)]
mod tests_parse {
    use super::*;

    #[test]
    fn a_f32() {
        let mut input = ["-1.03"].iter();
        let some: f32 = ParseFromIter::parse(&mut input).unwrap();
        assert_eq!(some, -1.03f32)
    }

    mod tuple_u32_u32_u32_u32 {
        use super::*;

        #[test]
        fn a_tuple_of_u32_u32_u32_u32() {
            let mut input = ["03579", "0", "1", "1"].iter();
            let some: (u32, u32, u32, u32) = ParseFromIter::parse(&mut input).unwrap();
            assert_eq!(some, (3579u32, 0u32, 1u32, 1u32))
        }

        #[test]
        #[should_panic]
        fn a_tuple_of_u32_u32_u32_u32_fail_on_neg() {
            let mut input = ["03579", "-0", "1", "1"].iter();
            let _some: (u32, u32, u32, u32) = ParseFromIter::parse(&mut input).unwrap();
        }

        #[test]
        #[should_panic]
        fn a_tuple_of_u32_u32_u32_u32_fail_on_length_too_short() {
            let mut input = ["03579", "-0", "1"].iter();
            let _some: (u32, u32, u32, u32) = ParseFromIter::parse(&mut input).unwrap();
        }

        #[test]
        #[should_panic]
        fn a_tuple_of_u32_u32_u32_u32_fail_not_u32() {
            let mut input = ["03579", "0", "1", "o"].iter();
            let _some: (u32, u32, u32, u32) = ParseFromIter::parse(&mut input).unwrap();
        }
    }

    mod array_f32x9 {
        use super::*;

        #[test]
        fn array_of_f32() {
            let mut input = ["1", "2", "3", "4", "5.5", "-6.0", "7", "8", "-9.9999"].iter();
            let some: [F32; 9] = ParseFromIter::parse(&mut input).unwrap();
            let expected: [F32; 9] = [
                F32(1f32),
                F32(2f32),
                F32(3f32),
                F32(4f32),
                F32(5.5f32),
                F32(-6.0f32),
                F32(7f32),
                F32(8f32),
                F32(-9.9999f32),
            ];
            assert_eq!(some, expected);
        }

        #[test]
        #[should_panic]
        fn array_of_f32_oof_too_short() {
            let mut input = ["1", "2", "3", "4", "5.5", "-6.0", "7", "8"].iter();
            let _some: [F32; 9] = ParseFromIter::parse(&mut input).unwrap();
        }

        #[test]
        #[should_panic]
        fn array_of_f32_oof_not_a_f32() {
            let mut input = ["1", "2", "3", "4", "5.5", "-6.0", "7", "8", "lalala"].iter();
            let _some: [F32; 9] = ParseFromIter::parse(&mut input).unwrap();
        }
    }

    mod tuple_u32_u32 {
        use super::*;

        #[test]
        fn a_tuple_of_u32_u32() {
            let mut input = ["03579", "0"].iter();
            let some: (u32, u32) = ParseFromIter::parse(&mut input).unwrap();
            assert_eq!(some, (3579u32, 0u32))
        }

        #[test]
        #[should_panic]
        fn a_tuple_of_u32_u32_fail_on_neg() {
            let mut input = ["03579", "-0"].iter();
            let _some: (u32, u32) = ParseFromIter::parse(&mut input).unwrap();
        }

        #[test]
        #[should_panic]
        fn a_tuple_of_u32_u32_fail_on_empty() {
            let input: [&str; 0] = [];
            let mut input = input.iter();
            let _some: (u32, u32) = ParseFromIter::parse(&mut input).unwrap();
        }

        #[test]
        #[should_panic]
        fn a_tuple_of_u32_u32_fail_on_too_short() {
            let mut input = ["03579"].iter();
            let _some: (u32, u32) = ParseFromIter::parse(&mut input).unwrap();
        }
    }

    mod tuple_f32_i32 {
        use super::*;

        #[test]
        fn a_tuple_of_f32_i32() {
            let mut input = ["-03579.1", "-1"].iter();
            let some: (F32, i32) = ParseFromIter::parse(&mut input).unwrap();
            assert_eq!(some, (F32(-3579.1f32), -1i32))
        }

        #[test]
        #[should_panic]
        fn a_tuple_of_f32_i32_fail_on_not_f32() {
            let mut input = ["f", "-1"].iter();
            let _some: (F32, i32) = ParseFromIter::parse(&mut input).unwrap();
        }

        #[test]
        #[should_panic]
        fn a_tuple_of_f32_i32_fail_on_not_i32() {
            let mut input = ["-1.0", "f"].iter();
            let _some: (F32, i32) = ParseFromIter::parse(&mut input).unwrap();
        }

        #[test]
        #[should_panic]
        fn a_tuple_of_f32_i32_fail_on_empty() {
            let empty: &[&str; 0] = &[];
            let mut empty = empty.iter();
            let _some: (F32, i32) = ParseFromIter::parse(&mut empty).unwrap();
        }

        #[test]
        #[should_panic]
        fn a_tuple_of_f32_i32_fail_on_too_short() {
            let mut input = ["03579"].iter();
            let _some: (F32, i32) = ParseFromIter::parse(&mut input).unwrap();
        }
    }

}
