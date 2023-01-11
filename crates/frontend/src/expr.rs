use codegem::ir::{ModuleBuilder, Type, Value};
use miette::Result;
use nom::{branch::alt, IResult, Parser};

use crate::{
    bin_op::{BinOp, Factor},
    function::Function,
    literal::Literal,
    var::Var,
    var_assign::VarAssign,
    Context, GetType, LowerToCodegem, Parse,
};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Literal(Literal),
    VarAssign(VarAssign),
    Function(Function),
    Factor(Factor),
    BinOp(BinOp),
    Var(Var),
}

impl Parse for Expr {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            // These must go first because they use keywords, which could be interpreted as identifiers
            VarAssign::parse.map(Self::VarAssign),
            Function::parse.map(Self::Function),
            //
            Var::parse.map(Self::Var),
            BinOp::parse.map(Self::BinOp),
            Factor::parse.map(Self::Factor),
            Literal::parse.map(Self::Literal),
        ))(input)
    }
}

impl LowerToCodegem for Expr {
    fn lower_to_code_gem(
        &self,
        builder: &mut ModuleBuilder,
        context: &mut Context,
    ) -> Result<Option<Value>> {
        match self {
            Expr::Literal(literal) => literal.lower_to_code_gem(builder, context),
            Expr::Var(var) => var.lower_to_code_gem(builder, context),
            Expr::VarAssign(var_assign) => var_assign.lower_to_code_gem(builder, context),
            Expr::Function(function) => function.lower_to_code_gem(builder, context),
            Expr::Factor(factor) => factor.lower_to_code_gem(builder, context),
            Expr::BinOp(bin_op) => bin_op.lower_to_code_gem(builder, context),
        }
    }
}

impl GetType for Expr {
    fn get_type(&self) -> Result<Type> {
        Ok(match self {
            Expr::Literal(l) => l.get_type()?,
            Expr::VarAssign(v) => v.get_type()?,
            Expr::Function(f) => f.get_type()?,
            Expr::Factor(f) => f.get_type()?,
            Expr::BinOp(b) => b.get_type()?,
            Expr::Var(v) => v.get_type()?,
        })
    }
}
