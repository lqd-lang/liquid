use std::{collections::HashMap, str::FromStr};

use codegem::ir::Linkage;
use frontend::node::NodeValue;
use lang_pt::ASTNode;
use lqdc_common::{codepass::CodePass, type_::Type, IntoLabelled, ScopeType, Stack};

use crate::parsepass::ParsePass;

pub struct MakeSignaturesPass<'input> {
    pub(crate) functions: HashMap<
        &'input str,
        (
            Linkage,
            Vec<(&'input str, Type)>,
            Type,
            Vec<ASTNode<NodeValue>>,
        ),
    >,
    scope: Stack<ScopeType>,
}

impl<'input> CodePass<'input> for MakeSignaturesPass<'input> {
    type Prev = ParsePass;
    type Arg = ();

    fn pass(mut previous: Self::Prev, input: &'input str, _: Self::Arg) -> miette::Result<Self> {
        let mut me = Self {
            functions: HashMap::new(),
            scope: Stack::new(),
        };
        me.run(&mut previous.nodes, input)?;
        Ok(me)
    }
}
impl<'input> MakeSignaturesPass<'input> {
    fn run(
        &mut self,
        nodes: &mut Vec<ASTNode<NodeValue>>,
        input: &'input str,
    ) -> miette::Result<()> {
        for node in nodes {
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
                NodeValue::FnDef => {
                    let id = &node.children[0];
                    let id = &input[id.start..id.end];

                    let linkage =
                        if self.scope.iter().any(|e| e == &ScopeType::Extern) || id == "main" {
                            Linkage::Public
                        } else {
                            Linkage::Private
                        };

                    let arg_nodes = &node.children[1];
                    let mut arg_nodes_iter = arg_nodes.children.iter();
                    let mut args = vec![];
                    while let Some(arg_name) = arg_nodes_iter.next() {
                        let type_node = arg_nodes_iter.next().unwrap();
                        let arg_name = &input[arg_name.start..arg_name.end];
                        let type_ = &input[type_node.start..type_node.end];
                        let type_ = Type::from_str(type_)
                            .map_err(|e| e.labelled((type_node.start..type_node.end).into()))?;
                        args.push((arg_name, type_));
                    }

                    let ret_type_node = &node.children[2];
                    let ret_type = &input[ret_type_node.start..ret_type_node.end];
                    let ret_type = Type::from_str(ret_type)
                        .map_err(|e| e.labelled((ret_type_node.start..ret_type_node.end).into()))?;

                    let nodes = node.children[3..node.children.len()].to_vec();

                    self.functions.insert(id, (linkage, args, ret_type, nodes));
                }
                NodeValue::FnCall => todo!(),
                NodeValue::FnDefArgSet => todo!(),
                NodeValue::FnCallArgSet => todo!(),
                NodeValue::Extern => {
                    self.scope.push(ScopeType::Extern);
                    self.run(&mut node.children, input)?;
                    self.scope.pop();
                }
                NodeValue::FnDecl => {
                    let id = &node.children[0];
                    let id = &input[id.start..id.end];

                    let linkage = if self.scope.iter().any(|e| e == &ScopeType::Extern) {
                        Linkage::External
                    } else {
                        Linkage::Private
                    };

                    let arg_nodes = &node.children[1];
                    let mut arg_nodes_iter = arg_nodes.children.iter();
                    let mut args = vec![];
                    while let Some(arg_name) = arg_nodes_iter.next() {
                        let type_node = arg_nodes_iter.next().unwrap();
                        let arg_name = &input[arg_name.start..arg_name.end];
                        let type_ = &input[type_node.start..type_node.end];
                        let type_ = Type::from_str(type_)
                            .map_err(|e| e.labelled((type_node.start..type_node.end).into()))?;
                        args.push((arg_name, type_));
                    }

                    let ret_type_node = &node.children[2];
                    let ret_type = &input[ret_type_node.start..ret_type_node.end];
                    let ret_type = Type::from_str(ret_type)
                        .map_err(|e| e.labelled((ret_type_node.start..ret_type_node.end).into()))?;

                    let nodes = node.children[3..node.children.len()].to_vec();

                    self.functions.insert(id, (linkage, args, ret_type, nodes));
                }
                NodeValue::BoolExpr => todo!(),
                NodeValue::True => todo!(),
                NodeValue::False => todo!(),
            }
        }

        Ok(())
    }
}
