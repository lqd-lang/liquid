use std::collections::HashMap;

use cranelift::prelude::{types::I64, AbiParam, FunctionBuilder, Type};
use cranelift_module::Module;
use lang_pt::ASTNode;
use miette::*;

use frontend::{node::NodeValue, parser};
use lqdc_common::{Error, IntoLabelled, ScopeType, Stack};

pub struct Compiler<'input, M: Module> {
    input: &'input str,
    scope: Stack<ScopeType>,
    types: HashMap<&'input str, Type>,
    module: M,
}

impl<'input, M: Module> Compiler<'input, M> {
    pub fn new(input: &'input str, module: M) -> Self {
        Self {
            input,
            scope: Stack::new(),
            types: HashMap::from([("int", I64)]),
            module,
        }
    }

    pub fn compile(&mut self) -> Result<()> {
        let parsed = {
            match parser().parse(self.input.as_bytes()) {
                Ok(parsed) => parsed,
                Err(parse_error) => {
                    bail!(Error::ParseError(parse_error.message,)
                        .labelled(parse_error.pointer.into()));
                }
            }
        };

        for node in &parsed {
            self.compile_top_level(node)?;
        }

        Ok(())
    }

    fn compile_top_level(&mut self, node: &ASTNode<NodeValue>) -> Result<()> {
        match node.node {
            NodeValue::Extern => {
                self.scope.push(ScopeType::Extern);
                for node in &node.children {
                    self.compile_top_level(node)?;
                }
                self.scope.pop();
                Ok(())
            }
            NodeValue::FnDecl => {
                let mut iter = node.children.iter();

                let id = iter.next().unwrap();
                let id = &self.input[id.start..id.end];

                let arg_nodes = iter.next().unwrap();
                let mut arg_nodes_iter = arg_nodes.children.iter();
                let mut args = vec![];
                while let Some(arg_name) = arg_nodes_iter.next() {
                    let type_node = arg_nodes_iter.next().unwrap();
                    let arg_name = &self.input[arg_name.start..arg_name.end];
                    let type_ = &self.input[type_node.start..type_node.end];
                    let type_ = self.types.get(type_).ok_or_else(|| {
                        Error::UnknownType.labelled((type_node.start..type_node.end).into())
                    })?;
                    args.push((arg_name.to_string(), type_.clone()));
                }

                let ret_type_node = iter.next().unwrap();
                let ret_type = &self.input[ret_type_node.start..ret_type_node.end];
                let ret_type = self.types.get(ret_type).ok_or_else(|| {
                    Error::UnknownReturnType
                        .labelled((ret_type_node.start..ret_type_node.end).into())
                })?;

                let mut signature = self.module.make_signature();
                for (_, type_) in args {
                    signature.params.push(AbiParam::new(type_));
                }
                signature.returns.push(AbiParam::new(*ret_type));

                todo!()
            }
            NodeValue::FnDef => todo!(),

            NodeValue::NULL
            | NodeValue::Id
            | NodeValue::Number
            | NodeValue::Add
            | NodeValue::Sub
            | NodeValue::Mul
            | NodeValue::Div
            | NodeValue::Product
            | NodeValue::Sum
            | NodeValue::Expr
            | NodeValue::Root
            | NodeValue::VarAssign
            | NodeValue::FnCall
            | NodeValue::FnDefArgSet
            | NodeValue::FnCallArgSet => unreachable!(),
        }
    }

    fn compile_node(
        &mut self,
        builder: &mut FunctionBuilder,
        node: &ASTNode<NodeValue>,
    ) -> Result<()> {
        match node.node {
            NodeValue::NULL => todo!(),
            NodeValue::Id => todo!(),
            NodeValue::Number => todo!(),
            NodeValue::Add => todo!(),
            NodeValue::Sub => todo!(),
            NodeValue::Mul => todo!(),
            NodeValue::Div => todo!(),
            NodeValue::Product => todo!(),
            NodeValue::Sum => todo!(),
            NodeValue::Expr => todo!(),
            NodeValue::Root => todo!(),
            NodeValue::VarAssign => todo!(),
            NodeValue::FnDef => todo!(),
            NodeValue::FnCall => todo!(),
            NodeValue::FnDefArgSet => todo!(),
            NodeValue::FnCallArgSet => todo!(),
            NodeValue::Extern => unreachable!(),
            NodeValue::FnDecl => unreachable!(),
        }
    }
}
