use nom::{branch::alt, character::complete::i64, Parser};

use crate::Parse;

#[derive(PartialEq, Debug)]
pub enum Literal {
    Int(i64),
}

impl Parse for Literal {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        alt((
            i64::parse.map(|x| Self::Int(x)),
            // Duplicate to satisfy compiler
            i64::parse.map(|x| Self::Int(x)),
        ))(input)
    }
}

impl Parse for i64 {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        i64(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::Parse;

    use super::Literal;

    #[test]
    fn test_parse_i64() {
        let input = "5476332";
        assert_eq!(Literal::parse(input).unwrap().1, Literal::Int(5476332));
    }
}
