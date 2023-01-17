use std::collections::{HashMap, VecDeque};

use codegem::ir::{
    BasicBlockId, FunctionId, Linkage, ModuleBuilder, Operation, Type, Value, VariableId,
};
use lang_pt::ASTNode;
use miette::*;

use frontend::{node::NodeValue, parser};

use crate::{Error, IntoLabelled};

lazy_static! {
    static ref GLOBAL_TYPES: HashMap<&'static str, Type> =
        HashMap::from([("int", Type::Integer(true, 64)), ("void", Type::Void)]);
}

pub struct Compiler<'a> {
    input: &'a str,
    main_function: Option<FunctionId>,
    main_function_entry_block: Option<BasicBlockId>,
    types: Types<'a>,
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
            functions: HashMap::new(),
        }
    }

    pub fn compile(&mut self, builder: &mut ModuleBuilder) -> Result<()> {
        let parser = parser();
        // TODO: Better error handling
        let parsed: Vec<ASTNode<NodeValue>> = match parser.parse(self.input.as_bytes()) {
            Ok(parsed) => parsed,
            Err(parse_error) => {
                bail!(Error::ParseError(parse_error.message,).labelled(parse_error.pointer.into()));
            }
        };

        self.main_function =
            Some(builder.new_function("__lqd_main__", Linkage::Public, &[], &Type::Void));
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

        let mut global_vars = HashMap::new();

        let mut compile_queue = VecDeque::new();
        for child in &root.children {
            self.compile_node(builder, child, &mut compile_queue, &mut global_vars)
                .map_err(|error| error.with_source_code(self.input.to_string()))?;
        }

        // Work on queue
        while !compile_queue.is_empty() {
            let (func_id, block_id, nodes, mut vars) = compile_queue.pop_front().unwrap();
            builder.switch_to_function(func_id);
            builder.switch_to_block(block_id);

            for node in nodes {
                self.compile_node(builder, &node, &mut compile_queue, &mut vars)
                    .map_err(|error| error.with_source_code(self.input.to_string()))?;
            }
        }

        builder.switch_to_function(self.main_function.unwrap());
        builder.switch_to_block(self.main_function_entry_block.unwrap());

        // Call main function
        ensure!(self.functions.contains_key("main"), "No main function");
        let main_function = self.functions.get("main").unwrap();
        ensure!(
            match main_function.0 {
                Type::Void => true,
                _ => false,
            },
            "Main function must return void"
        );

        builder.push_instruction(Operation::Call(main_function.1, vec![]));

        Ok(())
    }

    fn compile_node(
        &mut self,
        builder: &mut ModuleBuilder,
        node: &ASTNode<NodeValue>,
        compile_queue: &mut VecDeque<(
            FunctionId,
            BasicBlockId,
            Vec<ASTNode<NodeValue>>,
            HashMap<String, (Type, VariableId)>,
        )>,
        vars: &mut HashMap<String, (Type, VariableId)>,
    ) -> Result<Option<Value>> {
        match node.node {
            NodeValue::Id => {
                let id = &self.input[node.start..node.end];
                let (_, var_id) = if let Some(thing) = vars.get(id) {
                    thing
                } else {
                    bail!(Error::VarDoesntExist(id.to_string(),)
                        .labelled((node.start..node.end).into()))
                };
                Ok(builder.push_instruction(Operation::GetVar(*var_id)))
            }
            NodeValue::Number => {
                let num = self.input[node.start..node.end].parse::<i64>().unwrap();
                Ok(builder.push_instruction(Operation::Integer(
                    self.types.get("int").unwrap().clone(),
                    num.to_le_bytes().to_vec(),
                )))
            }
            NodeValue::Product => {
                if node.children.len() == 1 {
                    // Just a number
                    self.compile_node(builder, node.children.first().unwrap(), compile_queue, vars)
                } else if node.children.len() % 2 == 1 {
                    // dbg!(node.children.len());
                    let mut iter = node.children.iter();

                    let lhs = iter.next().unwrap();
                    let mut lhs_imm = self
                        .compile_node(builder, lhs, compile_queue, vars)?
                        .unwrap();
                    while let Some(op) = iter.next() {
                        let rhs = iter.next().unwrap();

                        // let lhs_imm = self.compile_node(builder, lhs)?.unwrap();
                        let rhs_imm = self
                            .compile_node(builder, rhs, compile_queue, vars)?
                            .unwrap();
                        lhs_imm = match op.node {
                            NodeValue::Mul => builder
                                .push_instruction(Operation::Mul(lhs_imm, rhs_imm))
                                .unwrap(),
                            NodeValue::Div => builder
                                .push_instruction(Operation::Div(lhs_imm, rhs_imm))
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
                    self.compile_node(builder, node.children.first().unwrap(), compile_queue, vars)
                } else if node.children.len() % 2 == 1 {
                    let mut iter = node.children.iter();

                    let mut result = None;
                    while let Some(node) = iter.next() {
                        let lhs = node;
                        let op = iter.next().unwrap();
                        let rhs = iter.next().unwrap();
                        let lhs_imm = self
                            .compile_node(builder, lhs, compile_queue, vars)?
                            .unwrap();
                        let rhs_imm = self
                            .compile_node(builder, rhs, compile_queue, vars)?
                            .unwrap();
                        let _lhs_type = self.type_of(lhs, vars);
                        result = match op.node {
                            NodeValue::Add => {
                                builder.push_instruction(Operation::Add(lhs_imm, rhs_imm))
                            }
                            NodeValue::Sub => {
                                builder.push_instruction(Operation::Sub(lhs_imm, rhs_imm))
                            }
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
                    result = self.compile_node(builder, child, compile_queue, vars)?;
                }
                Ok(result)
            }
            NodeValue::VarAssign => {
                let id = &node.children[0];
                let id = &self.input[id.start..id.end];
                let value = &node.children[1];
                let value_imm = self
                    .compile_node(builder, value, compile_queue, vars)?
                    .unwrap();
                let type_ = self.type_of(value, vars).clone();
                let var_id = builder.push_variable(id, &type_).unwrap();
                let result = builder.push_instruction(Operation::SetVar(var_id, value_imm));
                vars.insert(id.to_string(), (type_, var_id));
                Ok(result)
            }
            NodeValue::FnDef => {
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

                // create function
                let func_id = builder.new_function(id, Linkage::Private, args.as_slice(), ret_type);
                builder.switch_to_function(func_id);
                let block_id = builder.push_block().unwrap();
                builder.switch_to_block(block_id);

                let mut vars = HashMap::new();
                let func_args = builder.get_function_args(func_id);
                if func_args.is_some() {
                    let func_args = func_args.unwrap();
                    ensure!(
                        func_args.len() == args.len(),
                        Error::InternalCompilerError(
                            "func_args.len() != args.len()\nAdding function arguments failed"
                                .to_string()
                        )
                    );
                    for (variable_id, (name, type_)) in func_args.iter().zip(args) {
                        vars.insert(name, (type_, *variable_id));
                    }
                }

                self.functions.insert(id, (ret_type.clone(), func_id));

                // Add to compile queue
                compile_queue.push_back((
                    func_id,
                    block_id,
                    node.children[3..node.children.len()].to_vec(),
                    vars,
                ));

                // while let Some(node) = iter.next() {
                //     self.compile_node(builder, node)?;
                // }

                // switch back to main function
                builder.switch_to_function(self.main_function.unwrap());
                builder.switch_to_block(self.main_function_entry_block.unwrap());

                Ok(None)
            }
            NodeValue::FnCall => {
                let id = &node.children[0];
                let id = &self.input[id.start..id.end];

                let mut args = vec![];
                let arg_set = &node.children[1];
                match arg_set.node {
                    NodeValue::FnCallArgSet => {
                        for arg in &arg_set.children {
                            args.push(
                                self.compile_node(builder, arg, compile_queue, vars)?
                                    .ok_or_else(|| {
                                        Error::NotAllowedHere(
                                            format!("{node:?}"),
                                            "function calls".to_string(),
                                        )
                                        .labelled((arg.start..arg.end).into())
                                    })?,
                            )
                        }
                    }
                    NodeValue::NULL => {}
                    _ => unreachable!(),
                }

                let (_, function_id) = self
                    .functions
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

                Ok(builder.push_instruction(Operation::Call(*function_id, args)))
            }
            NodeValue::Add
            | NodeValue::Sub
            | NodeValue::Mul
            | NodeValue::Div
            | NodeValue::NULL
            | NodeValue::Root
            | NodeValue::FnDefArgSet
            | NodeValue::FnCallArgSet => {
                unreachable!()
            }
        }
    }

    fn type_of(
        &self,
        node: &ASTNode<NodeValue>,
        vars: &'a mut HashMap<String, (Type, VariableId)>,
    ) -> &Type {
        match node.node {
            NodeValue::Number => self.types.get("int").unwrap(),
            // Type of left hand side
            NodeValue::Sum => self.type_of(&node.children[0], vars),
            NodeValue::Product => self.type_of(&node.children[0], vars),
            NodeValue::Expr => self.type_of(node.children.last().unwrap(), vars),
            // Retrieve from variable list
            // It can be unwrapped, because it will already have been compiled, thus already checked
            NodeValue::Id => &vars.get(&self.input[node.start..node.end]).unwrap().0,
            a => {
                println!("{a:?} returned Void");
                &Type::Void
            }
        }
    }
}
