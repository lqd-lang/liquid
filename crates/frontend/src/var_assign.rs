use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
};

use crate::{identifier::Identifier, Expr, Parse};

#[derive(PartialEq, Debug)]
pub struct VarAssign {
    id: Identifier,
    val: Box<Expr>,
}

impl Parse for VarAssign {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let input = tag("let")(input)?.0;
        let input = multispace1(input)?.0;
        let (input, id) = Identifier::parse(input)?;
        let input = multispace0(input)?.0;
        let input = tag("=")(input)?.0;
        let input = multispace0(input)?.0;
        let (input, val) = Expr::parse(input)?;

        Ok((
            input,
            Self {
                id,
                val: Box::new(val),
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{identifier::Identifier, literal::Literal, Expr, Parse};

    use super::VarAssign;

    #[test]
    fn test_var_assign() {
        let input = "let fifteen = 15";
        let var_assign = VarAssign::parse(input).unwrap().1;
        assert_eq!(var_assign.id, Identifier("fifteen".to_string()));
        assert_eq!(var_assign.val, Box::new(Expr::Literal(Literal::Int(15))));
    }
}
