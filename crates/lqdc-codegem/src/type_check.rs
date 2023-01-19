use crate::parsepass::ParsePass;
use frontend::node::NodeValue;
use lqdc_common::codepass::CodePass;

pub struct TypeCheck;
impl<'input> CodePass<'input> for TypeCheck {
    type Prev = ParsePass;
    type Arg = ();

    fn check(previous: Self::Prev, _input: &str, _: Self::Arg) -> miette::Result<Self::Prev> {
        for node in &previous.nodes {
            match node.node {
                NodeValue::NULL => todo!(),
                NodeValue::Id => todo!(),
                NodeValue::Number => todo!(),
                NodeValue::Add => todo!(),
                NodeValue::Sub => todo!(),
                NodeValue::Mul => todo!(),
                NodeValue::Div => todo!(),
                NodeValue::GT => todo!(),
                NodeValue::GTE => todo!(),
                NodeValue::EQ => todo!(),
                NodeValue::LT => todo!(),
                NodeValue::LTE => todo!(),
                NodeValue::Product => todo!(),
                NodeValue::Sum => todo!(),
                NodeValue::Expr => todo!(),
                NodeValue::Root => todo!(),
                NodeValue::VarAssign => todo!(),
                NodeValue::FnDef => todo!(),
                NodeValue::FnCall => todo!(),
                NodeValue::FnDefArgSet => todo!(),
                NodeValue::FnCallArgSet => todo!(),
                NodeValue::Extern => todo!(),
                NodeValue::FnDecl => todo!(),
                NodeValue::BoolExpr => todo!(),
                NodeValue::True => todo!(),
                NodeValue::False => todo!(),
            }
        }

        Ok(previous)
    }
}
