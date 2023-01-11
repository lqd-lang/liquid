mod identifier;
mod literal;
mod var_assign;

use literal::Literal;
use nom::{branch::alt, IResult, Parser};
use var_assign::VarAssign;

trait Parse: Sized {
    fn parse(input: &str) -> IResult<&str, Self>;
}

#[derive(Debug, PartialEq)]
enum Expr {
    Literal(Literal),
    VarAssign(VarAssign),
}

impl Parse for Expr {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            Literal::parse.map(|x| Self::Literal(x)),
            VarAssign::parse.map(|x| Self::VarAssign(x)),
        ))(input)
    }
}
