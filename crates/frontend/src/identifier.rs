use nom::bytes::complete::take_while1;

use crate::Parse;

#[derive(Debug, PartialEq)]
pub struct Identifier(pub String);

impl Parse for Identifier {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (input, id) = take_while1(|x: char| x.is_alphabetic() || x == '_')(input)?;
        Ok((input, Self(id.to_string())))
    }
}
