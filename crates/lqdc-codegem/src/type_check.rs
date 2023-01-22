use std::collections::HashMap;

use codegem::ir::ModuleBuilder;
use lang_pt::ASTNode;
use miette::*;

use crate::make_signatures::MakeSignaturesPass;
use frontend::node::NodeValue;
use lqdc_common::{
    codepass::{CodePass, Is},
    type_::Type,
    Error, IntoLabelled,
};

pub struct TypeCheck<'input, 'a> {
    input: &'input str,
    // builder: &'input ModuleBuilder,
    prev: &'a MakeSignaturesPass<'input>,
    vars: HashMap<&'input str, Type>,
}
impl<'input, 'a> CodePass<'input> for TypeCheck<'input, 'a> {
    type Prev = MakeSignaturesPass<'input>;
    type Arg = &'input mut ModuleBuilder;

    fn check(prev: Self::Prev, input: &str, _builder: &impl Is<Self::Arg>) -> Result<Self::Prev> {
        let mut me = TypeCheck {
            input,
            // builder: builder.is(),
            prev: &prev,
            vars: HashMap::new(),
        };
        for function in &prev.functions {
            me.vars = HashMap::new();
            for (name, type_) in &function.1 .1 {
                me.vars.insert(name, *type_);
            }
            for node in &function.1 .3 {
                me.check_node(node)?;
            }
        }
        println!("Type checking complete");

        Ok(prev)
    }
}

impl TypeCheck<'_, '_> {
    fn check_node(&mut self, node: &ASTNode<NodeValue>) -> Result<Type> {
        match node.node {
            NodeValue::NULL => todo!(),
            NodeValue::Id => {
                let id = &self.input[node.start..node.end];
                Ok(self
                    .vars
                    .get(id)
                    .ok_or_else(|| {
                        Error::VarDoesntExist(id.to_string())
                            .labelled((node.start..node.end).into())
                    })
                    .cloned()?)
            }
            NodeValue::Number => Ok(Type::Int),
            NodeValue::Add => todo!(),
            NodeValue::Sub => todo!(),
            NodeValue::Mul => todo!(),
            NodeValue::Div => todo!(),
            NodeValue::GT => todo!(),
            NodeValue::GTE => todo!(),
            NodeValue::EQ => todo!(),
            NodeValue::LT => todo!(),
            NodeValue::LTE => todo!(),
            NodeValue::Product => {
                if node.children.len() == 1 {
                    self.check_node(&node.children[0])
                } else {
                    let mut iter = node.children.iter();

                    let lhs = self.check_node(iter.next().unwrap())?;
                    while let Some(_op) = iter.next() {
                        let rhs = self.check_node(iter.next().unwrap())?;

                        ensure!(lhs == rhs, "Mismatched types");
                    }

                    Ok(lhs)
                }
            }
            NodeValue::Sum => {
                if node.children.len() == 1 {
                    self.check_node(&node.children[0])
                } else {
                    let mut iter = node.children.iter();

                    let lhs = self.check_node(iter.next().unwrap())?;
                    while let Some(_op) = iter.next() {
                        let rhs = self.check_node(iter.next().unwrap())?;

                        ensure!(lhs == rhs, "Mismatched types");
                    }

                    Ok(lhs)
                }
            }
            NodeValue::Expr => self.check_node(&node.children[0]),
            NodeValue::Root => todo!(),
            NodeValue::VarAssign => todo!(),
            NodeValue::FnDef => todo!(),
            NodeValue::FnCall => {
                let id = &node.children[0];
                let id = &self.input[id.start..id.end];

                let mut args = vec![];
                let arg_set = &node.children[1];
                match arg_set.node {
                    NodeValue::FnCallArgSet => {
                        // dbg!(&arg_set.children);
                        for arg in &arg_set.children {
                            args.push((self.check_node(arg)?, arg.start, arg.end))
                        }
                    }
                    NodeValue::NULL => {}
                    _ => unreachable!(),
                }

                let (_, expected_args, ret_type, _) = self
                    .prev
                    .functions
                    .get(id)
                    .ok_or_else(|| miette!("Unknown function"))?;

                ensure!(
                    expected_args.len() == args.len(),
                    Error::ExpectedNumArgs(expected_args.len(), args.len())
                        .labelled((arg_set.start..arg_set.end).into())
                );

                for (expected, arg) in expected_args.iter().zip(args) {
                    ensure!(
                        expected.1 == arg.0,
                        Error::TypeMismatch(
                            format!("{:?}", expected.1),
                            format!("{:?}", arg.0),
                            (arg.1..arg.2).into()
                        )
                    )
                }

                Ok(*ret_type)
            }
            NodeValue::FnDefArgSet => todo!(),
            NodeValue::FnCallArgSet => todo!(),
            NodeValue::Extern => todo!(),
            NodeValue::FnDecl => todo!(),
            NodeValue::BoolExpr => {
                if node.children.len() == 1 {
                    self.check_node(&node.children[0])
                } else {
                    let mut iter = node.children.iter();

                    let lhs = self.check_node(iter.next().unwrap())?;
                    while let Some(_op) = iter.next() {
                        let rhs = self.check_node(iter.next().unwrap())?;

                        ensure!(lhs == rhs, "Mismatched types");
                    }

                    Ok(lhs)
                }
            }
            NodeValue::True => Ok(Type::Bool),
            NodeValue::False => Ok(Type::Bool),
        }
    }
}
