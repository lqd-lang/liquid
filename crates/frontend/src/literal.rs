use codegem::ir::{Operation, Type};
use miette::Result;
use nom::{branch::alt, character::complete::i64, Parser};

use crate::{GetType, LowerToCodegem, Parse};

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

impl GetType for Literal {
    fn get_type(&self) -> Result<Type> {
        Ok(match self {
            Literal::Int(_) => Type::Integer(true, 4),
        })
    }
}

impl LowerToCodegem for Literal {
    fn lower_to_code_gem(
        &self,
        builder: &mut codegem::ir::ModuleBuilder,
    ) -> Result<Option<codegem::ir::Value>> {
        match self {
            Literal::Int(num) => Ok(builder.push_instruction(
                &Type::Integer(true, 64),
                Operation::Integer(true, num.to_le_bytes().to_vec()),
            )),
        }
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
