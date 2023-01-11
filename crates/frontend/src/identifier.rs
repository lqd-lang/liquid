use nom::character::complete::alpha1;

use crate::Parse;

#[derive(Debug, PartialEq)]
pub struct Identifier(pub String);

impl Parse for Identifier {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (input, id) = alpha1(input)?;
        Ok((input, Self(id.to_string())))
    }
}
