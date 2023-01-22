use std::collections::HashMap;

use lang_pt::ASTNode;
use miette::*;

use crate::{
    codepass::{CodePass, Is},
    make_signatures::MakeSignaturesPass,
    type_::Type,
    Error, IntoLabelled,
};
use frontend::node::NodeValue;

pub struct TypeCheck<'input, 'a> {
    input: &'input str,
    // builder: &'input ModuleBuilder,
    prev: &'a MakeSignaturesPass<'input>,
    vars: HashMap<&'input str, Type>,
}
impl<'input, 'a> CodePass<'input> for TypeCheck<'input, 'a> {
    type Prev = MakeSignaturesPass<'input>;
    type Arg = ();

    fn check(prev: Self::Prev, input: &str, _: &impl Is<Self::Arg>) -> Result<Self::Prev> {
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
            let mut result = Type::Void;
            for node in &function.1 .3 {
                result = me.check_node(node)?;
            }
            ensure!(
                result == function.1 .2,
                Error::TypeMismatch(
                    format!("{:?}", function.1 .2),
                    format!("{:?}", result),
                    (function.1 .3.last().unwrap().start..function.1 .3.last().unwrap().end).into()
                )
            )
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

                    Ok(Type::Bool)
                }
            }
            NodeValue::True => Ok(Type::Bool),
            NodeValue::False => Ok(Type::Bool),
        }
    }
}

#[cfg(test)]
mod tests {
    use miette::*;

    use crate::{codepass::PassRunner, make_signatures::MakeSignaturesPass, parsepass::ParsePass};

    use super::TypeCheck;

    #[test]
    fn bool_in_int_function_call() -> Result<()> {
        let input = "
        fn main -> void {
            other_function(false);
        }

        fn other_function(x: int) -> int {
            x
        }
        ";
        let result = PassRunner::<(), ()>::new(input)
            .run::<ParsePass>()?
            .run::<MakeSignaturesPass>()?
            .inject::<TypeCheck>();

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn int_in_bool_function_call() -> Result<()> {
        let input = "
        fn main -> void {
            other_function(1);
        }

        fn other_function(x: bool) -> bool {
            x
        }
        ";
        let result = PassRunner::<(), ()>::new(input)
            .run::<ParsePass>()?
            .run::<MakeSignaturesPass>()?
            .inject::<TypeCheck>();

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn return_specified_type() -> Result<()> {
        let input = "
        fn main -> void {
            true
        }
        fn other_function -> bool {
            14
        }
        fn otherer_function -> int {
            false
        }
        ";
        let result = PassRunner::<(), ()>::new(input)
            .run::<ParsePass>()?
            .run::<MakeSignaturesPass>()?
            .inject::<TypeCheck>();

        assert!(result.is_err());

        Ok(())
    }
}
