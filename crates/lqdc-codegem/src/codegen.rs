use std::collections::HashMap;

use codegem::ir::{FunctionId, ModuleBuilder, Operation, ToIntegerOperation, Value, VariableId};
use lang_pt::ASTNode;
use miette::*;

use crate::{map_linkage, map_type, CodegemError};
use frontend::node::NodeValue;
use lqdc_common::{
    codepass::{CodePass, Is},
    linkage::Linkage,
    make_signatures::MakeSignaturesPass,
    type_::Type,
    Error, IntoLabelled,
};

pub struct CodegenPass;
impl<'input> CodePass<'input> for CodegenPass {
    type Prev = MakeSignaturesPass<'input>;
    type Arg = &'input mut ModuleBuilder;

    fn pass(
        prev: Self::Prev,
        input: &'input str,
        builder: &mut impl Is<Self::Arg>,
    ) -> miette::Result<Self> {
        let mut functions = HashMap::new();
        let builder = builder.is_mut();
        for (name, (linkage, args, ret_type, nodes)) in prev.functions {
            let func_id = builder.new_function(
                name,
                map_linkage(&linkage),
                args.iter()
                    .map(|(a, t)| (a.to_string(), map_type(*t)))
                    .collect::<Vec<_>>()
                    .as_slice(),
                &map_type(ret_type),
            );
            functions.insert(name, (linkage, args, ret_type, nodes, func_id));
        }
        for (_, (_, args, _, nodes, func_id)) in &functions {
            let mut vars = HashMap::new();
            for ((name, type_), id) in args
                .iter()
                .zip(builder.get_function_args(*func_id).unwrap())
            {
                vars.insert(name.to_string(), (*type_, id));
            }
            builder.switch_to_function(*func_id);
            let block = builder
                .push_block()
                .map_err(CodegemError::ModuleCreationError)?;
            builder.switch_to_block(block);
            for node in nodes {
                compile_node(input, builder, node, &mut vars, &functions)?;
            }
        }

        Ok(Self)
    }
}

fn compile_node(
    input: &str,
    builder: &mut ModuleBuilder,
    node: &ASTNode<NodeValue>,
    vars: &mut HashMap<String, (Type, VariableId)>,
    functions: &HashMap<
        &str,
        (
            Linkage,
            Vec<(&str, Type)>,
            Type,
            Vec<ASTNode<NodeValue>>,
            FunctionId,
        ),
    >,
) -> Result<Option<Value>> {
    match node.node {
        NodeValue::Id => {
            let id = &input[node.start..node.end];
            let (_, var_id) = if let Some(thing) = vars.get(id) {
                thing
            } else {
                bail!(Error::VarDoesntExist(id.to_string(),).labelled((node.start..node.end).into()))
            };
            Ok(builder
                .push_instruction(Operation::GetVar(*var_id))
                .map_err(CodegemError::ModuleCreationError)?)
        }
        NodeValue::Number => {
            let num = input[node.start..node.end].parse::<i64>().unwrap();
            Ok(builder
                .push_instruction(Operation::Integer(
                    map_type(Type::Int),
                    num.to_le_bytes().to_vec(),
                ))
                .map_err(CodegemError::ModuleCreationError)?)
        }
        NodeValue::Product => {
            if node.children.len() == 1 {
                // Just a number
                compile_node(
                    input,
                    builder,
                    node.children.first().unwrap(),
                    vars,
                    functions,
                )
            } else if node.children.len() % 2 == 1 {
                // dbg!(node.children.len());
                let mut iter = node.children.iter();

                let lhs = iter.next().unwrap();
                let mut lhs_imm = compile_node(input, builder, lhs, vars, functions)?.unwrap();
                while let Some(op) = iter.next() {
                    let rhs = iter.next().unwrap();

                    // let lhs_imm = compile_node(input,builder,lhs)?.unwrap();
                    let rhs_imm = compile_node(input, builder, rhs, vars, functions)?.unwrap();
                    lhs_imm = match op.node {
                        NodeValue::Mul => builder
                            .push_instruction(Operation::Mul(lhs_imm, rhs_imm))
                            .map_err(CodegemError::ModuleCreationError)?
                            .unwrap(),
                        NodeValue::Div => builder
                            .push_instruction(Operation::Div(lhs_imm, rhs_imm))
                            .map_err(CodegemError::ModuleCreationError)?
                            .unwrap(),
                        _ => unreachable!(),
                    };
                }
                Ok(Some(lhs_imm))
            } else {
                bail!(miette!("Even number of arguments"))
            }
        }
        NodeValue::Sum => {
            if node.children.len() == 1 {
                compile_node(
                    input,
                    builder,
                    node.children.first().unwrap(),
                    vars,
                    functions,
                )
            } else if node.children.len() % 2 == 1 {
                let mut iter = node.children.iter();

                let mut result = None;
                while let Some(node) = iter.next() {
                    let lhs = node;
                    let op = iter.next().unwrap();
                    let rhs = iter.next().unwrap();
                    let lhs_imm = compile_node(input, builder, lhs, vars, functions)?.unwrap();
                    let rhs_imm = compile_node(input, builder, rhs, vars, functions)?.unwrap();
                    let _lhs_type = type_of(input, lhs, vars, functions);
                    result = match op.node {
                        NodeValue::Add => builder
                            .push_instruction(Operation::Add(lhs_imm, rhs_imm))
                            .map_err(CodegemError::ModuleCreationError)?,
                        NodeValue::Sub => builder
                            .push_instruction(Operation::Sub(lhs_imm, rhs_imm))
                            .map_err(CodegemError::ModuleCreationError)?,
                        _ => unreachable!(),
                    };
                }

                Ok(result)
            } else {
                unreachable!()
            }
        }
        NodeValue::Expr => {
            let mut result = None;
            for child in &node.children {
                result = compile_node(input, builder, child, vars, functions)?;
            }
            Ok(result)
        }
        NodeValue::VarAssign => {
            let id = &node.children[0];
            let id = &input[id.start..id.end];
            let value = &node.children[1];
            let value_imm = compile_node(input, builder, value, vars, functions)?.unwrap();
            let type_ = type_of(input, value, vars, functions);
            let var_id = builder.push_variable(id, &map_type(type_)).unwrap();
            let result = builder.push_instruction(Operation::SetVar(var_id, value_imm));
            vars.insert(id.to_string(), (type_, var_id));
            Ok(result.map_err(CodegemError::ModuleCreationError)?)
        }
        NodeValue::FnCall => {
            let id = &node.children[0];
            let id = &input[id.start..id.end];

            let mut args = vec![];
            let arg_set = &node.children[1];
            match arg_set.node {
                NodeValue::FnCallArgSet => {
                    // dbg!(&arg_set.children);
                    for arg in &arg_set.children {
                        args.push(
                            compile_node(input, builder, arg, vars, functions)?.ok_or_else(
                                || {
                                    Error::NotAllowedHere(
                                        format!("{:?}", arg.node),
                                        "function calls".to_string(),
                                    )
                                    .labelled((arg.start..arg.end).into())
                                },
                            )?,
                        )
                    }
                }
                NodeValue::NULL => {}
                _ => unreachable!(),
            }

            let (_, _, _, _, function_id) = functions
                .get(id)
                .ok_or_else(|| miette!("Unknown function"))?;
            let func_args = builder.get_function_args(*function_id).ok_or_else(|| {
                Error::InternalCompilerError(
                    "Failed to get function, invalid function_id".to_string(),
                )
            })?;
            ensure!(
                func_args.len() == args.len(),
                Error::ExpectedNumArgs(func_args.len(), args.len())
                    .labelled((arg_set.start..arg_set.end).into())
            );

            Ok(builder
                .push_instruction(Operation::Call(*function_id, args))
                .map_err(CodegemError::ModuleCreationError)?)
        }
        NodeValue::BoolExpr => {
            if node.children.len() == 1 {
                compile_node(input, builder, &node.children[0], vars, functions)
            } else {
                let mut iter = node.children.iter();

                let mut result = None;
                while let Some(lhs) = iter.next() {
                    let op = iter.next().unwrap();
                    let rhs = iter.next().unwrap();
                    // dbg!(lhs);
                    // dbg!(op);
                    // dbg!(rhs);
                    let lhs_imm = compile_node(input, builder, lhs, vars, functions)?.unwrap();
                    let rhs_imm = compile_node(input, builder, rhs, vars, functions)?.unwrap();
                    let _lhs_type = type_of(input, lhs, vars, functions);
                    result = match op.node {
                        NodeValue::GT => builder
                            .push_instruction(Operation::Gt(lhs_imm, rhs_imm))
                            .map_err(CodegemError::ModuleCreationError)?,
                        NodeValue::GTE => builder
                            .push_instruction(Operation::Ge(lhs_imm, rhs_imm))
                            .map_err(CodegemError::ModuleCreationError)?,
                        NodeValue::EQ => builder
                            .push_instruction(Operation::Eq(lhs_imm, rhs_imm))
                            .map_err(CodegemError::ModuleCreationError)?,
                        NodeValue::LT => builder
                            .push_instruction(Operation::Lt(lhs_imm, rhs_imm))
                            .map_err(CodegemError::ModuleCreationError)?,
                        NodeValue::LTE => builder
                            .push_instruction(Operation::Le(lhs_imm, rhs_imm))
                            .map_err(CodegemError::ModuleCreationError)?,
                        _ => unreachable!(),
                    };
                }

                Ok(result)
            }
        }
        NodeValue::True => Ok(builder
            .push_instruction(0b1_u8.to_integer_operation())
            .map_err(CodegemError::ModuleCreationError)?),
        NodeValue::False => Ok(builder
            .push_instruction(0b0_u8.to_integer_operation())
            .map_err(CodegemError::ModuleCreationError)?),
        NodeValue::Add
        | NodeValue::Sub
        | NodeValue::Mul
        | NodeValue::Div
        | NodeValue::NULL
        | NodeValue::Root
        | NodeValue::FnDefArgSet
        | NodeValue::FnCallArgSet
        | NodeValue::GT
        | NodeValue::GTE
        | NodeValue::EQ
        | NodeValue::LT
        | NodeValue::LTE
        | NodeValue::FnDef
        | NodeValue::FnDecl
        | NodeValue::Extern => {
            unreachable!()
        }
    }
}

fn type_of(
    input: &str,
    node: &ASTNode<NodeValue>,
    vars: &mut HashMap<String, (Type, VariableId)>,
    functions: &HashMap<
        &str,
        (
            Linkage,
            Vec<(&str, Type)>,
            Type,
            Vec<ASTNode<NodeValue>>,
            FunctionId,
        ),
    >,
) -> Type {
    match node.node {
        NodeValue::Number => Type::Int,
        // Type of left hand side
        NodeValue::Sum => type_of(input, &node.children[0], vars, functions),
        NodeValue::Product => type_of(input, &node.children[0], vars, functions),
        NodeValue::Expr => type_of(input, node.children.last().unwrap(), vars, functions),
        // Retrieve from variable list
        // It can be unwrapped, because it will already have been compiled, thus already checked
        NodeValue::Id => vars.get(&input[node.start..node.end]).unwrap().0,
        NodeValue::True => Type::Bool,
        NodeValue::False => Type::Bool,
        a => {
            dbg!(a);
            Type::Void
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(debug_assertions)]
    mod type_of {
        use std::collections::HashMap;

        use frontend::parser;
        use lqdc_common::type_::Type;

        use crate::codegen::type_of;

        #[test]
        fn true_() {
            let type_ = type_of(
                "",
                parser()
                    .debug_production_at("value", "true".as_bytes(), 0)
                    .unwrap()
                    .first()
                    .unwrap(),
                &mut HashMap::new(),
                &mut HashMap::new(),
            );
            assert_eq!(type_, Type::Bool,);
        }

        #[test]
        fn false_() {
            let type_ = type_of(
                "",
                parser()
                    .debug_production_at("value", "false".as_bytes(), 0)
                    .unwrap()
                    .first()
                    .unwrap(),
                &mut HashMap::new(),
                &mut HashMap::new(),
            );
            assert_eq!(type_, Type::Bool);
        }

        #[test]
        fn number() {
            let type_ = type_of(
                "",
                parser()
                    .debug_production_at("value", "158910".as_bytes(), 0)
                    .unwrap()
                    .first()
                    .unwrap(),
                &mut HashMap::new(),
                &mut HashMap::new(),
            );
            assert_eq!(type_, Type::Int);
        }
    }
}
