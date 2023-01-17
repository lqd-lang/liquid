mod func_compiler;
mod node;

use std::collections::HashMap;

use cranelift::{
    codegen::ir::{FuncRef, Function, SigRef, UserExternalName, UserFuncName},
    prelude::{isa::CallConv, settings::Flags, types::*, *},
};
use cranelift_module::{Linkage, Module, ModuleError};
use cranelift_object::{ObjectBuilder, ObjectModule};

use frontend::{node::NodeValue, parser};
use lang_pt::ASTNode;
use miette::*;

use crate::{
    cranelift::func_compiler::{FuncCompiler, Program},
    Error, IntoLabelled,
};

lazy_static! {
    static ref GLOBAL_TYPES: HashMap<&'static str, Type> =
        HashMap::from([("int", I64), ("void", types::I8)]);
}

#[derive(Debug, Diagnostic, Error)]
pub enum CraneliftError {
    #[error("Unknown target '{}'", .0)]
    UnknownTarget(String),
    #[error(transparent)]
    ModuleError(ModuleError),
}

#[derive(Debug, Default)]
pub struct SymbolTable<'a> {
    inner: HashMap<&'a str, UserExternalName>,
    idx: u32,
}

impl<'a> SymbolTable<'a> {
    pub fn get(&self, id: &'a str) -> Option<UserExternalName> {
        self.inner.get(id).map(|name| name.clone())
    }

    pub fn insert(&mut self, id: &'a str) -> UserExternalName {
        let name = self.allocate();
        self.insert_specific(id, name.clone());
        name
    }

    pub fn insert_specific(&mut self, id: &'a str, name: UserExternalName) {
        self.inner.insert(id, name);
    }

    /// Create a name, without adding it to inner.
    /// Used when the function is not yet available.
    pub fn allocate(&mut self) -> UserExternalName {
        let name = UserExternalName::new(0, self.idx);
        self.idx += 1;
        name
    }
}

#[derive(Debug)]
pub struct Compiler<'a, M: Module> {
    input: &'a str,
    types: Types<'a>,
    functions: HashMap<&'a str, (FuncRef, SigRef)>,
    current_vars: Option<HashMap<&'a str, (u32, Type)>>,
    current_func: Option<&'a str>,
    module_name: &'a str,
    module: Option<M>,
    symbol_table: SymbolTable<'a>,
}

#[derive(Debug, Default)]
pub struct Types<'a> {
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

impl<'a, M: Module> Compiler<'a, M> {
    pub fn new(input: &'a str, module_name: &'a str) -> Self {
        Self {
            input,
            types: Types::default(),
            functions: HashMap::new(),
            module_name,
            module: None,
            current_vars: None,
            current_func: None,
            symbol_table: SymbolTable::default(),
        }
    }

    pub fn compile(&mut self) -> Result<()> {
        ensure!(
            self.module.is_some(),
            "Module not initialised, try calling Compiler::use_object_module()"
        );

        let root = {
            let parser = parser();
            match parser.parse(self.input.as_bytes()) {
                Ok(parsed) => parsed,
                Err(parse_error) => {
                    bail!(Error::ParseError(parse_error.message,)
                        .labelled(parse_error.pointer.into()));
                }
            }
            .remove(0)
        };

        let mut program = Program {
            input: self.input,
            types: Types::default(),
            symbol_table: SymbolTable::default(),
            fn_builder_ctx: FunctionBuilderContext::new(),
        };
        for func in root.children {
            // self.compile_node(
            //     &mut FunctionBuilder::new(&mut Function::new(), &mut FunctionBuilderContext::new()),
            //     func,
            // )
            // .map_err(|error| error.with_source_code(self.input.to_string()))?;
            let mut func_compiler =
                FuncCompiler::new(&mut program, self.module.as_mut().unwrap(), func.children)?;
            func_compiler.compile()?;
        }

        Ok(())
    }

    fn compile_node(
        &mut self,
        builder: &mut FunctionBuilder,
        node: &ASTNode<NodeValue>,
    ) -> Result<Option<Value>> {
        match node.node {
            NodeValue::NULL => todo!(),
            NodeValue::Id => {
                let var = &self.input[node.start..node.end];
                let (var, _type) =
                    self.current_vars
                        .as_ref()
                        .unwrap()
                        .get(var)
                        .ok_or_else(|| {
                            Error::VarDoesntExist(var.to_string())
                                .labelled((node.start..node.end).into())
                        })?;
                let var = Variable::from_u32(*var);
                Ok(Some(builder.use_var(var)))
            }
            NodeValue::Number => {
                let num = &self.input[node.start..node.end];
                let num = num
                    .parse::<i64>()
                    .map_err(|_| Error::InvalidInteger.labelled((node.start..node.end).into()))?;

                Ok(Some(builder.ins().iconst(I64, num)))
            }
            NodeValue::Add => todo!(),
            NodeValue::Sub => todo!(),
            NodeValue::Mul => todo!(),
            NodeValue::Div => todo!(),
            NodeValue::Product => {
                ensure!(
                    node.children.len() > 0,
                    Error::InternalCompilerError(format!(
                        "Product node with no children\n{}",
                        node
                    ))
                );
                if node.children.len() == 1 {
                    self.compile_node(builder, node.children.first().unwrap())
                } else {
                    ensure!(
                        node.children.len() % 2 == 1,
                        Error::InternalCompilerError(
                            "Product node with even number of children".to_string()
                        )
                    );

                    let mut iter = node.children.iter();

                    let mut lhs = self.compile_node(builder, iter.next().unwrap())?.unwrap();
                    while let Some(op) = iter.next() {
                        let rhs = iter.next().unwrap();
                        let rhs = self.compile_node(builder, rhs)?.unwrap();

                        lhs = match op.node {
                            NodeValue::Mul => builder.ins().imul(lhs, rhs),
                            NodeValue::Div => builder.ins().fdiv(lhs, rhs),
                            _ => unreachable!(),
                        };
                    }
                    Ok(Some(lhs))
                }
            }
            NodeValue::Sum => {
                ensure!(
                    node.children.len() > 0,
                    Error::InternalCompilerError("Sum node with no children".to_string())
                );
                if node.children.len() == 1 {
                    self.compile_node(builder, node.children.first().unwrap())
                } else {
                    ensure!(
                        node.children.len() % 2 == 1,
                        Error::InternalCompilerError(
                            "Sum node with even number of children".to_string()
                        )
                    );

                    let mut iter = node.children.iter();

                    let mut lhs = self.compile_node(builder, iter.next().unwrap())?.unwrap();
                    while let Some(op) = iter.next() {
                        let rhs = iter.next().unwrap();
                        let rhs = self.compile_node(builder, rhs)?.unwrap();

                        lhs = match op.node {
                            NodeValue::Add => builder.ins().iadd(lhs, rhs),
                            NodeValue::Sub => builder.ins().isub(lhs, rhs),
                            _ => unreachable!(),
                        };
                    }
                    Ok(Some(lhs))
                }
            }
            NodeValue::Expr => {
                let mut result = None;
                for node in &node.children {
                    result = self.compile_node(builder, node)?;
                }
                Ok(result)
            }
            NodeValue::Root => unreachable!(),
            NodeValue::VarAssign => {
                let mut iter = node.children.iter();

                let id = iter.next().unwrap();
                let id = &self.input[id.start..id.end];

                let value_node = iter.next().unwrap();
                let value = self.compile_node(builder, value_node)?.unwrap();

                let var_id = self
                    .current_vars
                    .as_ref()
                    .unwrap()
                    .values()
                    .last()
                    .unwrap_or(&(0, *self.types.get("void").unwrap()))
                    .0
                    + 1;
                let variable = Variable::from_u32(var_id);
                let type_ = self.type_of(builder, value_node)?.unwrap();
                builder.declare_var(variable, type_);
                builder.def_var(variable, value);
                self.current_vars
                    .as_mut()
                    .unwrap()
                    .insert(id, (var_id, type_));

                Ok(None)
            }
            NodeValue::FnDef => {
                let mut iter = node.children.iter();

                let id = iter.next().unwrap();
                let id = &self.input[id.start..id.end];

                let mut signature = Signature::new(CallConv::SystemV);

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
                    signature.params.push(AbiParam::new(*type_));
                    args.push((arg_name, type_.clone()));
                }

                let ret_type_node = iter.next().unwrap();
                let ret_type = &self.input[ret_type_node.start..ret_type_node.end];
                let ret_type = self.types.get(ret_type).ok_or_else(|| {
                    Error::UnknownReturnType
                        .labelled((ret_type_node.start..ret_type_node.end).into())
                })?;
                signature.returns.push(AbiParam::new(*ret_type));

                let func_id = {
                    let module = self.module.as_mut().unwrap();
                    module
                        .declare_function(id, Linkage::Local, &signature)
                        .map_err(CraneliftError::ModuleError)?
                };
                let sig_ref = builder.import_signature(signature.clone());

                let name = self.symbol_table.insert(id);
                let mut function =
                    Function::with_name_signature(UserFuncName::User(name), signature);
                let mut fn_builder_ctx = FunctionBuilderContext::new();

                let mut mbuilder = FunctionBuilder::new(&mut function, &mut fn_builder_ctx);

                let entry_block = mbuilder.create_block();
                mbuilder.append_block_params_for_function_params(entry_block);

                mbuilder.switch_to_block(entry_block);

                self.current_vars = Some(HashMap::new());
                self.current_func = Some(id);
                let mut result = None;
                for node in iter {
                    result = self.compile_node(&mut mbuilder, node)?;
                }

                ensure!(
                    result.is_some(),
                    Error::InternalCompilerError("Result is none".to_string())
                );

                mbuilder.ins().return_(&[result.unwrap()]);

                mbuilder.seal_all_blocks();
                mbuilder.finalize();

                {
                    let module = self.module.as_mut().unwrap();
                    let func_ref = module.declare_func_in_func(func_id, &mut function);
                    self.functions.insert(id, (func_ref, sig_ref));
                }

                Ok(None)
            }
            NodeValue::FnCall => {
                let mut iter = node.children.iter();

                let func_node = iter.next().unwrap();
                let func_name = &self.input[func_node.start..func_node.end];
                let (_func_ref, sig_ref) = self.functions.get(func_name).ok_or_else(|| {
                    Error::FuncDoesntExist(func_name.to_string())
                        .labelled((func_node.start..func_node.end).into())
                })?;

                let name = self.symbol_table.get(func_name).ok_or_else(|| {
                    Error::FuncDoesntExist(func_name.to_string())
                        .labelled((func_node.start..func_node.end).into())
                })?;
                let name_ref = builder.func.declare_imported_user_function(name);
                let func_ref = builder.import_function(ExtFuncData {
                    name: ExternalName::User(name_ref),
                    signature: *sig_ref,
                    colocated: false,
                });

                println!("Before call");
                builder.ins().call(func_ref, &[]);
                println!("After call");

                todo!()
            }
            NodeValue::FnDefArgSet => todo!(),
            NodeValue::FnCallArgSet => todo!(),
        }
    }

    fn type_of(
        &self,
        builder: &mut FunctionBuilder,
        node: &ASTNode<NodeValue>,
    ) -> Result<Option<Type>> {
        match node.node {
            NodeValue::NULL => todo!(),
            NodeValue::Id => {
                let id = &self.input[node.start..node.end];
                let (_var, type_) =
                    self.current_vars.as_ref().unwrap().get(id).ok_or_else(|| {
                        Error::VarDoesntExist(id.to_string())
                            .labelled((node.start..node.end).into())
                    })?;

                Ok(Some(*type_))
            }
            NodeValue::Number => Ok(self.types.get("int").map(|t| *t)),
            NodeValue::Product | NodeValue::Sum | NodeValue::Expr => {
                self.type_of(builder, node.children.first().unwrap())
            }
            NodeValue::Root => unreachable!(),
            NodeValue::VarAssign => todo!(),
            NodeValue::FnDef => todo!(),
            NodeValue::FnCall => {
                let id_node = node.children.first().unwrap();
                let id = &self.input[id_node.start..id_node.end];
                let (_, sig_ref) = self.functions.get(id).ok_or_else(|| {
                    Error::FuncDoesntExist(id.to_string())
                        .labelled((id_node.start..id_node.end).into())
                })?;
                let signature = builder.signature(*sig_ref).unwrap();

                Ok(Some(signature.returns.first().unwrap().value_type))
            }
            NodeValue::Add
            | NodeValue::Sub
            | NodeValue::Mul
            | NodeValue::Div
            | NodeValue::FnCallArgSet
            | NodeValue::FnDefArgSet => unreachable!(),
        }
    }
}

impl<'a> Compiler<'a, ObjectModule> {
    pub fn use_object_module(&mut self, target: &str) -> Result<()> {
        let obj_builder = ObjectBuilder::new(
            isa::lookup_by_name(target)
                .map_err(|_| CraneliftError::UnknownTarget(target.to_string()))?
                .finish(Flags::new(settings::builder()))
                .unwrap(),
            self.module_name,
            cranelift_module::default_libcall_names(),
        )
        .unwrap();
        let module = ObjectModule::new(obj_builder);
        self.module = Some(module);

        Ok(())
    }
}
