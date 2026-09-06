mod ast;
mod codegen;
mod parser;

use clap::Parser;
use inkwell::context::Context;

#[derive(Parser)]
#[command(about = "arithmetic expression compiler via LLVM")]
struct Args {
    expr: String,
    #[arg(short = 'O', long, help = "run optimization passes before executing")]
    optimize: bool,
}

fn main() {
    let args = Args::parse();
    let expr = parser::parse(&args.expr).unwrap_or_else(|e| {
        eprintln!("parse error: {e}");
        std::process::exit(1);
    });

    let context = Context::create();
    let cg = codegen::CodeGen::new(&context);
    cg.compile(&expr);
    if args.optimize {
        cg.optimize();
    }
    cg.print_ir();
    cg.jit_run();
}
