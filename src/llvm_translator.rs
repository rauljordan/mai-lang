use std::collections::HashMap;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::passes::PassManager;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValue,FloatValue,FunctionValue,PointerValue};
use inkwell::FloatPredicate;

use crate::parser::*;
use crate::token::Token;

pub struct Translator<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub fpm: &'a PassManager<FunctionValue<'ctx>>,
    pub module: &'a Module<'ctx>,
    pub variables: HashMap<String, PointerValue<'ctx>>,
    pub fn_value_opt: Option<FunctionValue<'ctx>>,
}

impl<'a, 'ctx> Translator<'a, 'ctx> {
    fn create_stack_alloc(&self, name: &str) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let entry = self.fn_value_opt.unwrap().get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.context.f64_type(), name)
    }

    pub fn translate_function_sig(&self, fun: &Stmt) -> Result<FunctionValue<'ctx>, &'static str> {
        let Stmt::Function { name: Token::Ident(fn_name), params, body: _ } = fun else {
            panic!("Not a function");
        };
        let return_type = self.context.f64_type();
        let arg_types = std::iter::repeat(return_type)
            .take(params.len())
            .map(|f| f.into())
            .collect::<Vec<BasicMetadataTypeEnum>>();
        let args = arg_types.as_slice();

        let fn_type = self.context.f64_type().fn_type(args, false); // No var args.
        let fn_val = self.module.add_function(fn_name.as_str(), fn_type, None);

        for (i, arg) in fn_val.get_param_iter().enumerate() {
            let param = params[i].clone();
            let Token::Ident(arg_ident) = param else {
                panic!("Not an arg ident");
            };
            arg.into_float_value().set_name(arg_ident.as_str());
        }

        Ok(fn_val)
    }

    pub fn translate_function(&mut self, fun: &Stmt) -> Result<FunctionValue<'ctx>, &'static str> {
        let Stmt::Function { name: _, params, body } = fun else {
            panic!("Not a function");
        };
        let sig = self.translate_function_sig(fun)?;
        if body.is_empty() {
            return Ok(sig);
        }
        let entry = self.context.append_basic_block(sig, "entry");
        self.builder.position_at_end(entry);
        self.fn_value_opt = Some(sig);
        self.variables.reserve(params.len());

        for (i, arg) in sig.get_param_iter().enumerate() {
            let param = params[i].clone();
            let Token::Ident(arg_ident) = param else {
                panic!("Not an arg ident");
            };
            let alloca = self.create_stack_alloc(arg_ident.as_str());
            self.builder.build_store(alloca, arg);
            self.variables.insert(arg_ident, alloca);
        }

        let body = self.translate_stmt(body.first().unwrap())?;

        self.builder.build_return(Some(&body));

        if sig.verify(true) {
            self.fpm.run_on(&sig);
            return Ok(sig);
        }
        unsafe {
            sig.delete();
        }

        Err("Invalid generated function")
    }

    fn translate_stmt(&mut self, stmt: &Box<Stmt>) -> Result<FloatValue<'ctx>, &'static str> {
        match stmt.as_ref() {
            Stmt::Expr(expr) => self.translate_expr(expr),
            Stmt::If { 
                cond, 
                then_branch, 
                else_branch,
            } => self.translate_conditional(cond, then_branch, else_branch),
            Stmt::Block(statements) => {
                return self.translate_stmt(statements.first().unwrap());
            }
            Stmt::Return { keyword: _, value } => {
                if value.is_some() {
                    let value = value.as_ref().unwrap();
                    return self.translate_expr(value);
                }
                return Ok(self.context.f64_type().const_zero());
            },
            item => panic!("Could not handle value: {:?}", item)
        }
    }

    pub fn translate_conditional(
        &mut self,
        cond: &Box<Expr>, 
        then_branch: &Box<Stmt>, 
        else_branch: &Option<Box<Stmt>>
    ) -> Result<FloatValue<'ctx>, &'static str> {
        let parent = self.fn_value_opt.unwrap();
        let zero_const = self.context.f64_type().const_float(0.0);

        // create condition by comparing without 0.0 and returning an int
        let cond = self.translate_expr(cond)?;
        let cond = self
            .builder
            .build_float_compare(FloatPredicate::ONE, cond, zero_const, "ifcond");

        // build branch
        let then_bb = self.context.append_basic_block(parent, "then");
        let else_bb = self.context.append_basic_block(parent, "else");
        let cont_bb = self.context.append_basic_block(parent, "ifcont");

        self.builder.build_conditional_branch(cond, then_bb, else_bb);

        // build then block
        self.builder.position_at_end(then_bb);
        let then_val = self.translate_stmt(then_branch)?;
        self.builder.build_unconditional_branch(cont_bb);

        let then_bb = self.builder.get_insert_block().unwrap();

        let mut incoming: Vec<(&dyn BasicValue, BasicBlock)> = vec![];
        incoming.push((&then_val, then_bb));

        // build else block
        self.builder.position_at_end(else_bb);
        let else_b = *else_branch.clone().unwrap();
        let else_box = Box::new(else_b);
        let else_val = self.translate_stmt(&else_box)?;
        let else_bb = self.builder.get_insert_block().unwrap();
        self.builder.build_unconditional_branch(cont_bb);
        incoming.push((&else_val, else_bb));

        // emit merge block
        self.builder.position_at_end(cont_bb);

        let phi = self.builder.build_phi(self.context.f64_type(), "iftmp");
        phi.add_incoming(incoming.as_slice());

        Ok(phi.as_basic_value().into_float_value())
    }

    pub fn translate_expr(&self, expr: &Box<Expr>) -> Result<FloatValue<'ctx>, &'static str> {
        match expr.as_ref() {
            Expr::Literal{ value: nb } => {
                let f: f64 = nb.parse::<f64>().unwrap();
                Ok(self.context.f64_type().const_float(f))
            },
            Expr::Variable { name } => {
                let Token::Ident(id) = name else {
                    panic!("Not an ident");
                };
                match self.variables.get(id.as_str()) {
                    Some(var) => Ok(self.builder.build_load(*var, id.as_str()).into_float_value()),
                    None => Err("Could not find a matching variable"),
                }
            },
            Expr::BinaryExpr {
                op,
                ref left,
                ref right,
            } => {
                    let lhs = self.translate_expr(left)?;
                    let rhs = self.translate_expr(right)?;

                    match op {
                        Token::Plus => Ok(self.builder.build_float_add(lhs, rhs, "tmpadd")),
                        Token::Minus => Ok(self.builder.build_float_sub(lhs, rhs, "tmpsub")),
                        Token::Less => Ok({
                            let cmp = self
                                .builder
                                .build_float_compare(FloatPredicate::ULT, lhs, rhs, "tmpcmp");

                            self.builder
                                .build_unsigned_int_to_float(cmp, self.context.f64_type(), "tmpbool")
                        }),
                        Token::Greater => Ok({
                            let cmp = self
                                .builder
                                .build_float_compare(FloatPredicate::ULT, rhs, lhs, "tmpcmp");

                            self.builder
                                .build_unsigned_int_to_float(cmp, self.context.f64_type(), "tmpbool")
                        }),
                        _ => Err("unsupported binary operation"),
                    }
            },
            _ => Err("unable to compile expression to LLVM")
        }
    }

    pub fn translate(
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        pass_manager: &'a PassManager<FunctionValue<'ctx>>,
        module: &'a Module<'ctx>,
        stmt: &Stmt,
    ) -> Result<FunctionValue<'ctx>, &'static str> {
        let mut tr = Translator {
            context,
            builder,
            fpm: pass_manager,
            module,
            fn_value_opt: None,
            variables: HashMap::new(),
        };

        tr.translate_function(stmt)
    }
}

