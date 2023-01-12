#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

use codegem::ir::{BasicBlockId, FunctionId, ModuleBuilder, Operation, Type, Value, VariableId};
use lang_pt::ASTNode;
use miette::{bail, miette, Result};

use frontend::{node::NodeValue, parser};

lazy_static! {
    static ref GLOBAL_TYPES: HashMap<&'static str, Type> =
        HashMap::from([("int", Type::Integer(true, 64)), ("void", Type::Void)]);
}

pub struct Compiler<'a> {
    input: &'a str,
    main_function: Option<FunctionId>,
    main_function_entry_block: Option<BasicBlockId>,
    types: Types<'a>,
    vars: HashMap<&'a str, (Type, VariableId)>,
    functions: HashMap<&'a str, (Type, FunctionId)>,
}

#[derive(Default)]
struct Types<'a> {
    inner: HashMap<&'a str, Type>,
}

impl Types<'_> {
    fn get(&self, key: &str) -> Option<&Type> {
        let res = self.inner.get(key);
        if res.is_none() {
            return GLOBAL_TYPES.get(key);
        }
        res
    }
}

impl<'a> Compiler<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            main_function: None,
            main_function_entry_block: None,
            types: Types::default(),
            vars: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn compile(&mut self, builder: &mut ModuleBuilder) -> Result<()> {
        let parser = parser();
        // TODO: Better error handling
        let parsed = parser.parse(self.input.as_bytes()).unwrap();

        self.main_function = Some(builder.new_function("__lqd_main__", &[], &Type::Void));
        builder.switch_to_function(self.main_function.unwrap());

        self.main_function_entry_block = builder.push_block();
        builder.switch_to_block(self.main_function_entry_block.unwrap());

        if parsed.len() != 1 {
            bail!(miette!("Failed to parse"))
        }

        let root = parsed.first().unwrap();
        if root.node != NodeValue::Root {
            bail!(miette!("Not root"))
        }

        for child in &root.children {
            self.compile_node(builder, child)?;
        }

        Ok(())
    }

    fn compile_node(
        &mut self,
        builder: &mut ModuleBuilder,
        node: &ASTNode<NodeValue>,
    ) -> Result<Option<Value>> {
        match node.node {
            NodeValue::Id => {
                let id = &self.input[node.start..node.end];
                let (type_, var_id) = if let Some(thing) = self.vars.get(id) {
                    thing
                } else {
                    bail!(miette!("Variable '{}' does not exist", id))
                };
                Ok(builder.push_instruction(type_, Operation::GetVar(*var_id)))
            }
            NodeValue::Number => {
                let num = self.input[node.start..node.end].parse::<i64>().unwrap();
                Ok(builder.push_instruction(
                    self.types.get("int").unwrap(),
                    Operation::Integer(true, num.to_le_bytes().to_vec()),
                ))
            }
            NodeValue::Product => {
                if node.children.len() == 1 {
                    // Just a number
                    self.compile_node(builder, node.children.first().unwrap())
                } else if node.children.len() % 2 == 1 {
                    // dbg!(node.children.len());
                    let mut iter = node.children.iter();

                    let lhs = iter.next().unwrap();
                    let mut lhs_imm = self.compile_node(builder, lhs)?.unwrap();
                    while let Some(op) = iter.next() {
                        let rhs = iter.next().unwrap();

                        // let lhs_imm = self.compile_node(builder, lhs)?.unwrap();
                        let rhs_imm = self.compile_node(builder, rhs)?.unwrap();
                        lhs_imm = match op.node {
                            NodeValue::Mul => builder
                                .push_instruction(
                                    self.type_of(lhs),
                                    Operation::Mul(lhs_imm, rhs_imm),
                                )
                                .unwrap(),
                            NodeValue::Div => builder
                                .push_instruction(
                                    self.type_of(lhs),
                                    Operation::Div(lhs_imm, rhs_imm),
                                )
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
                    self.compile_node(builder, node.children.first().unwrap())
                } else if node.children.len() % 2 == 1 {
                    let mut iter = node.children.iter();

                    while let Some(node) = iter.next() {
                        let lhs = node;
                        let op = iter.next().unwrap();
                        let rhs = iter.next().unwrap();
                        let lhs_imm = self.compile_node(builder, lhs)?.unwrap();
                        let rhs_imm = self.compile_node(builder, rhs)?.unwrap();
                        let lhs_type = self.type_of(lhs);
                        match op.node {
                            NodeValue::Add => {
                                builder.push_instruction(lhs_type, Operation::Add(lhs_imm, rhs_imm))
                            }
                            NodeValue::Sub => {
                                builder.push_instruction(lhs_type, Operation::Sub(lhs_imm, rhs_imm))
                            }
                            _ => unreachable!(),
                        };
                    }

                    Ok(None)
                } else {
                    unreachable!()
                }
            }
            NodeValue::Expr => {
                let mut result = None;
                for child in &node.children {
                    result = self.compile_node(builder, child)?;
                }
                Ok(result)
            }
            NodeValue::VarAssign => {
                let id = &node.children[0];
                let id = &self.input[id.start..id.end];
                let value = &node.children[1];
                let value_imm = self.compile_node(builder, value)?.unwrap();
                let type_ = self.type_of(value).clone();
                let var_id = builder.push_variable(id, &type_).unwrap();
                let result = builder.push_instruction(&type_, Operation::SetVar(var_id, value_imm));
                self.vars.insert(id, (type_, var_id));
                Ok(result)
            }
            NodeValue::FnDef => {
                let mut iter = node.children.iter();

                let id = iter.next().unwrap();
                let id = &self.input[id.start..id.end];

                let ret_type = iter.next().unwrap();
                let ret_type = &self.input[ret_type.start..ret_type.end];
                let ret_type = self
                    .types
                    .get(ret_type)
                    .ok_or_else(|| miette!("Invalid type"))?;

                // create function
                let func_id = builder.new_function(id, &[], ret_type);
                builder.switch_to_function(func_id);
                let block_id = builder.push_block().unwrap();
                builder.switch_to_block(block_id);

                self.functions.insert(id, (ret_type.clone(), func_id));

                while let Some(node) = iter.next() {
                    self.compile_node(builder, node)?;
                }

                // switch back to main function
                builder.switch_to_function(self.main_function.unwrap());
                builder.switch_to_block(self.main_function_entry_block.unwrap());

                Ok(None)
            }
            NodeValue::FnCall => {
                let id = &node.children[0];
                let id = &self.input[id.start..id.end];

                let (type_, function_id) = self
                    .functions
                    .get(id)
                    .ok_or_else(|| miette!("Unknown function"))?;
                Ok(builder.push_instruction(type_, Operation::Call(*function_id, vec![])))
            }
            NodeValue::Add
            | NodeValue::Sub
            | NodeValue::Mul
            | NodeValue::Div
            | NodeValue::NULL
            | NodeValue::Root => {
                unreachable!()
            }
        }
    }

    fn type_of(&self, node: &ASTNode<NodeValue>) -> &Type {
        match node.node {
            NodeValue::Number => self.types.get("int").unwrap(),
            // Type of left hand side
            NodeValue::Sum => self.type_of(&node.children[0]),
            NodeValue::Product => self.type_of(&node.children[0]),
            NodeValue::Expr => self.type_of(node.children.last().unwrap()),
            // Retrieve from variable list
            // It can be unwrapped, because it will already have been compiled, thus already checked
            NodeValue::Id => &self.vars.get(&self.input[node.start..node.end]).unwrap().0,
            a => {
                println!("{a:?} returned Void");
                &Type::Void
            }
        }
    }
}
