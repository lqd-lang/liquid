use cranelift::{
    codegen::ir::{Function, UserFuncName},
    prelude::*,
};
use cranelift_module::{Linkage, Module};
use frontend::node::NodeValue;
use lang_pt::ASTNode;

use miette::*;

use crate::{Error, IntoLabelled};

use super::{CraneliftError, SymbolTable, Types};

pub struct FuncCompiler<'module, 'config, 'input, 'types, 'symbols, 'builder, M: Module>
where
    'input: 'symbols,
    'input: 'builder,
    'config: 'builder,
    'types: 'builder,
{
    program: &'config mut Program<'input, 'types, 'symbols>,
    module: &'module mut M,
    nodes: Vec<ASTNode<NodeValue>>,
    builder: FunctionBuilder<'builder>,
    id: &'input str,
    signature: Signature,
}
// To share between functions
pub struct Program<'input, 'types, 'symbols> {
    pub input: &'input str,
    pub types: Types<'types>,
    pub symbol_table: SymbolTable<'symbols>,
    pub fn_builder_ctx: FunctionBuilderContext,
}

impl<'module, 'config, 'input, 'types, 'symbols, M: Module>
    FuncCompiler<'module, 'config, 'input, 'types, 'symbols, '_, M>
{
    pub fn new(
        program: &'config mut Program<'input, 'types, 'symbols>,
        module: &'module mut M,
        nodes: Vec<ASTNode<NodeValue>>,
    ) -> Result<Self> {
        ensure!(nodes.len() > 3, "Empty function");

        let mut iter = nodes.iter();

        let id_node = iter.next().unwrap();
        let id = &program.input[id_node.start..id_node.end];

        // TODO
        let _arg_set = iter.next().unwrap();

        let return_type_node = iter.next().unwrap();
        let return_type = &program.input[return_type_node.start..return_type_node.end];
        let return_type = program.types.get(return_type).ok_or_else(|| {
            Error::UnknownType.labelled((return_type_node.start..return_type_node.end).into())
        })?;

        let mut signature = module.make_signature();
        signature.returns.push(AbiParam::new(*return_type));

        let mut function = Function::with_name_signature(
            UserFuncName::User(program.symbol_table.insert(id)),
            signature,
        );
        let builder = FunctionBuilder::new(&mut function, &mut program.fn_builder_ctx);
        Ok(Self {
            program,
            module,
            nodes,
            builder,
            id,
            signature,
        })
    }

    pub fn compile(&mut self) -> Result<()> {
        let _func_id = self
            .module
            .declare_function(self.id, Linkage::Local, &self.signature)
            .map_err(|e| CraneliftError::ModuleError(e))?;

        let mut result = None;
        for node in &self.nodes[3..self.nodes.len()] {
            result = Some(self.compile_node(node)?);
        }

        self.builder.ins().return_(&[result.unwrap()]);

        Ok(())
    }

    pub fn finalize(self) {
        self.builder.finalize()
    }

    pub fn compile_node(&mut self, node: &ASTNode<NodeValue>) -> Result<Value> {
        match node.node {
            NodeValue::Id => todo!(),
            NodeValue::Number => {
                let num = &self.program.input[node.start..node.end];
                let num = num.parse::<isize>();

                todo!()
            }
            NodeValue::Product => {
                if node.children.len() == 1 {
                    self.compile_node(&node.children[0])
                } else {
                    ensure!(
                        node.children.len() % 2 == 1,
                        "Must have an even number of children"
                    );

                    let mut iter = node.children.iter();

                    let lhs = iter.next().unwrap();
                    let mut lhs = self.compile_node(lhs)?;
                    while let Some(op) = iter.next() {
                        let rhs = iter.next().map(|rhs| self.compile_node(rhs)).unwrap()?;

                        lhs = match op.node {
                            NodeValue::Mul => self.builder.ins().imul(lhs, rhs),
                            NodeValue::Div => self.builder.ins().fdiv(lhs, rhs),
                            _ => unreachable!(),
                        }
                    }

                    Ok(lhs)
                }
            }
            NodeValue::Sum => {
                if node.children.len() == 1 {
                    self.compile_node(&node.children[0])
                } else {
                    ensure!(
                        node.children.len() % 2 == 1,
                        "Must have an odd number of children"
                    );

                    let mut iter = node.children.iter();

                    let lhs = iter.next().unwrap();
                    let mut lhs = self.compile_node(lhs)?;
                    while let Some(op) = iter.next() {
                        let rhs = iter.next().map(|rhs| self.compile_node(rhs)).unwrap()?;

                        lhs = match op.node {
                            NodeValue::Add => self.builder.ins().iadd(lhs, rhs),
                            NodeValue::Sub => self.builder.ins().isub(lhs, rhs),
                            _ => unreachable!(),
                        }
                    }

                    Ok(lhs)
                }
            }
            NodeValue::Expr => {
                ensure!(node.children.len() == 1, "Expr can only have one child");

                self.compile_node(&node.children[0])
            }
            NodeValue::VarAssign => todo!(),
            NodeValue::FnDef => todo!(),
            NodeValue::FnCall => todo!(),
            // Unreachable
            NodeValue::NULL
            | NodeValue::Root
            | NodeValue::Add
            | NodeValue::Sub
            | NodeValue::Mul
            | NodeValue::Div
            | NodeValue::FnDefArgSet
            | NodeValue::FnCallArgSet => unreachable!(),
        }
    }
}
