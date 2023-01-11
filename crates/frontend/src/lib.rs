mod identifier;
mod literal;
mod var_assign;

use codegem::ir::{ModuleBuilder, Type, Value};
use identifier::Identifier;
use literal::Literal;

use miette::Result;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    IResult, Parser,
};
use var_assign::VarAssign;

pub trait Parse: Sized {
    fn parse(input: &str) -> IResult<&str, Self>;
}

pub trait LowerToCodegem {
    fn lower_to_code_gem(&self, builder: &mut ModuleBuilder) -> Result<Option<Value>>;
}

trait GetType {
    fn get_type(&self) -> Result<Type>;
}

#[derive(PartialEq, Debug)]
pub struct Function {
    id: Identifier,
    expr: Box<Expr>,
}

impl Parse for Function {
    fn parse(input: &str) -> IResult<&str, Self> {
        let input = tag("fn")(input)?.0;
        let input = multispace1(input)?.0;
        let (input, id) = Identifier::parse(input)?;
        let input = tag(":")(input)?.0;
        let input = multispace0(input)?.0;
        let (input, expr) = Expr::parse(input)?;

        Ok((
            input,
            Self {
                id,
                expr: Box::new(expr),
            },
        ))
    }
}

impl LowerToCodegem for Function {
    fn lower_to_code_gem(&self, builder: &mut ModuleBuilder) -> Result<Option<Value>> {
        let func_id = builder.new_function(&self.id.0, &[], &Type::Void);
        builder.switch_to_function(func_id);
        let entry_block = builder.push_block().expect("Failed to create entry block");
        builder.switch_to_block(entry_block);
        self.expr.lower_to_code_gem(builder)?;

        Ok(None)
    }
}

impl GetType for Function {
    fn get_type(&self) -> Result<Type> {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Literal(Literal),
    VarAssign(VarAssign),
    Function(Function),
}

impl Parse for Expr {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            Literal::parse.map(|x| Self::Literal(x)),
            VarAssign::parse.map(|x| Self::VarAssign(x)),
            Function::parse.map(|x| Self::Function(x)),
        ))(input)
    }
}

impl LowerToCodegem for Expr {
    fn lower_to_code_gem(&self, builder: &mut ModuleBuilder) -> Result<Option<Value>> {
        match self {
            Expr::Literal(literal) => literal.lower_to_code_gem(builder),
            Expr::VarAssign(var_assign) => var_assign.lower_to_code_gem(builder),
            Expr::Function(function) => function.lower_to_code_gem(builder),
        }
    }
}

impl GetType for Expr {
    fn get_type(&self) -> Result<Type> {
        Ok(match self {
            Expr::Literal(l) => l.get_type()?,
            Expr::VarAssign(v) => v.get_type()?,
            Expr::Function(f) => f.get_type()?,
        })
    }
}
