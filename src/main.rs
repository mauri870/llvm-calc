mod ast;
mod parser;

use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::values::FloatValue;

use ast::{BinOp, Expr};

fn compile_expr<'ctx>(expr: &Expr, ctx: &'ctx Context, builder: &Builder<'ctx>) -> FloatValue<'ctx> {
    match expr {
        Expr::Number(n) => ctx.f64_type().const_float(*n),
        Expr::BinOp { op, left, right } => {
            let l = compile_expr(left, ctx, builder);
            let r = compile_expr(right, ctx, builder);
            match op {
                BinOp::Add => builder.build_float_add(l, r, "add").unwrap(),
                BinOp::Sub => builder.build_float_sub(l, r, "sub").unwrap(),
                BinOp::Mul => builder.build_float_mul(l, r, "mul").unwrap(),
                BinOp::Div => builder.build_float_div(l, r, "div").unwrap(),
            }
        }
    }
}

fn main() {
    let input = std::env::args().nth(1).unwrap_or_else(|| "42".into());
    let expr = parser::parse(&input).unwrap_or_else(|e| {
        eprintln!("parse error: {e}");
        std::process::exit(1);
    });

    let context = Context::create();
    let module = context.create_module("calc");
    let builder = context.create_builder();

    let f64_type = context.f64_type();
    let fn_type = f64_type.fn_type(&[], false);
    let function = module.add_function("eval", fn_type, None);
    let entry = context.append_basic_block(function, "entry");
    builder.position_at_end(entry);
    let value = compile_expr(&expr, &context, &builder);
    builder.build_return(Some(&value)).unwrap();

    println!("=== IR ===");
    println!("{}", module.print_to_string().to_string());

    let ee = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    let result = unsafe {
        ee.get_function::<unsafe extern "C" fn() -> f64>("eval")
            .unwrap()
            .call()
    };
    println!("result: {result}");
}
