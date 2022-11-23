use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::passes::PassManager;
use inkwell::values::{FloatValue, FunctionValue};
use inkwell::FloatPredicate;

use crate::parser::Expr;
use crate::token::Token;

pub struct LLVMTranslator<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub fpm: &'a PassManager<FunctionValue<'ctx>>,
    pub module: &'a Module<'ctx>,
}

impl<'a, 'ctx> LLVMTranslator<'a, 'ctx> {
    fn translate_expr(&mut self, expr: &Expr) -> Result<FloatValue<'ctx>, &'static str> {
        match expr {
            Expr::Literal{ value: nb } => {
                let f: f64 = nb.parse::<f64>().unwrap();
                Ok(self.context.f64_type().const_float(f))
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
                        //Token::Eqq => Ok(self.builder.build_float_compare(FloatPredicate::OEQ, lhs, rhs, "tmpadd")),
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
        expr: &Expr,
    ) -> Result<FloatValue<'ctx>, &'static str> {
        let mut tr = LLVMTranslator {
            context,
            builder,
            fpm: pass_manager,
            module,
        };

        tr.translate_expr(expr)
    }
}

