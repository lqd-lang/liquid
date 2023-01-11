use codegem::ir::{Operation, Type, Value};
use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
};

use crate::{identifier::Identifier, Expr, GetType, LowerToCodegem, Parse};

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

impl LowerToCodegem for VarAssign {
    fn lower_to_code_gem(
        &self,
        builder: &mut codegem::ir::ModuleBuilder,
    ) -> miette::Result<Option<Value>> {
        let var_id = builder
            .push_variable(&self.id.0, &self.val.get_type()?)
            .expect("Failed to create variable");
        // ZERO
        // let value = builder
        //     .push_instruction(
        //         &Type::Integer(true, 4),
        //         Operation::Integer(true, 0_i64.to_le_bytes().to_vec()),
        //     )
        //     .unwrap();
        dbg!(self);
        let value = self.val.lower_to_code_gem(builder)?.unwrap();
        builder.push_instruction(&Type::Void, Operation::SetVar(var_id, value));

        Ok(None)
    }
}

impl GetType for VarAssign {
    fn get_type(&self) -> miette::Result<Type> {
        Ok(Type::Void)
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
