use codegem::ir::{ModuleBuilder, Operation, Type, Value};
use miette::miette;

use crate::{identifier::Identifier, Context, GetType, LowerToCodegem, Parse};

#[derive(Debug, PartialEq)]
pub struct Var {
    id: Identifier,
}

impl Parse for Var {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (input, id) = Identifier::parse(input)?;

        Ok((input, Self { id }))
    }
}
impl LowerToCodegem for Var {
    fn lower_to_code_gem(
        &self,
        builder: &mut ModuleBuilder,
        context: &mut Context,
    ) -> miette::Result<Option<Value>> {
        let (id, type_) = context
            .vars
            .get(&self.id.0)
            .ok_or_else(|| miette!("Var does not exist"))?;
        Ok(builder.push_instruction(type_, Operation::GetVar(*id)))
    }
}
impl GetType for Var {
    fn get_type(&self) -> miette::Result<Type> {
        todo!()
    }
}
