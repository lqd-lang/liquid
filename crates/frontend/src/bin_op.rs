use codegem::ir::{ModuleBuilder, Operation, Value};
use nom::{branch::alt, bytes::complete::tag, character::complete::multispace0, Parser};

use crate::{
    expr::Expr, function::Function, literal::Literal, var::Var, var_assign::VarAssign, Context,
    GetType, LowerToCodegem, Parse,
};

#[derive(Debug, PartialEq)]
pub struct BinOp {
    lhs: Box<Expr>,
    rhs: Box<Expr>,
    op: BinOpOp,
}

#[derive(Debug, PartialEq)]
pub enum BinOpOp {
    Add,
    Sub,
}

impl Parse for BinOpOp {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let add = tag("+").map(|_| Self::Add);
        let sub = tag("-").map(|_| Self::Sub);
        alt((add, sub))(input)
    }
}

impl Parse for BinOp {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (input, lhs) = alt((
            Factor::parse.map(Expr::Factor),
            Literal::parse.map(Expr::Literal),
            Var::parse.map(Expr::Var),
            Function::parse.map(Expr::Function),
            VarAssign::parse.map(Expr::VarAssign),
            BinOp::parse.map(Expr::BinOp),
        ))(input)?;
        let input = multispace0(input)?.0;
        let (input, op) = BinOpOp::parse(input)?;
        let input = multispace0(input)?.0;
        let (input, rhs) = Expr::parse(input)?;

        Ok((
            input,
            Self {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                op,
            },
        ))
    }
}

impl GetType for BinOp {
    fn get_type(&self) -> miette::Result<codegem::ir::Type> {
        self.lhs.get_type()
    }
}

impl LowerToCodegem for BinOp {
    fn lower_to_code_gem(
        &self,
        builder: &mut ModuleBuilder,
        context: &mut Context,
    ) -> miette::Result<Option<Value>> {
        let lhs = self.lhs.lower_to_code_gem(builder, context)?.unwrap();
        let rhs = self.rhs.lower_to_code_gem(builder, context)?.unwrap();
        match &self.op {
            BinOpOp::Add => {
                Ok(builder.push_instruction(&self.get_type()?, Operation::Add(lhs, rhs)))
            }
            BinOpOp::Sub => {
                Ok(builder.push_instruction(&self.get_type()?, Operation::Sub(lhs, rhs)))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Factor {
    lhs: Box<Expr>,
    rhs: Box<Expr>,
    op: FactorOp,
}

#[derive(Debug, PartialEq)]
pub enum FactorOp {
    Mul,
    Div,
}

impl Parse for FactorOp {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        alt((tag("*").map(|_| Self::Mul), tag("/").map(|_| Self::Div)))(input)
    }
}

impl Parse for Factor {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (input, lhs) = alt((
            Literal::parse.map(Expr::Literal),
            Var::parse.map(Expr::Var),
            Factor::parse.map(Expr::Factor),
            Function::parse.map(Expr::Function),
            VarAssign::parse.map(Expr::VarAssign),
            BinOp::parse.map(Expr::BinOp),
        ))(input)?;
        let input = multispace0(input)?.0;
        let (input, op) = FactorOp::parse(input)?;
        let input = multispace0(input)?.0;
        let (input, rhs) = Expr::parse(input)?;

        Ok((
            input,
            Self {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                op,
            },
        ))
    }
}

impl GetType for Factor {
    fn get_type(&self) -> miette::Result<codegem::ir::Type> {
        self.lhs.get_type()
    }
}

impl LowerToCodegem for Factor {
    fn lower_to_code_gem(
        &self,
        builder: &mut ModuleBuilder,
        context: &mut Context,
    ) -> miette::Result<Option<Value>> {
        let lhs = self.lhs.lower_to_code_gem(builder, context)?.unwrap();
        let rhs = self.rhs.lower_to_code_gem(builder, context)?.unwrap();
        Ok(match self.op {
            FactorOp::Mul => {
                builder.push_instruction(&self.lhs.get_type()?, Operation::Mul(lhs, rhs))
            }
            FactorOp::Div => {
                builder.push_instruction(&self.lhs.get_type()?, Operation::Div(lhs, rhs))
            }
        })
    }
}
