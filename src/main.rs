mod ast;
mod codegen;
mod parser;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use inkwell::context::Context;

#[derive(Parser)]
#[command(about = "arithmetic expression compiler via LLVM")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
    /// Expression to JIT-execute (default when no subcommand given)
    expr: Option<String>,
    #[arg(short = 'O', long, help = "run optimization passes")]
    optimize: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Print LLVM IR and exit
    Ir {
        expr: String,
        #[arg(short = 'O', long, help = "run optimization passes first")]
        optimize: bool,
        #[arg(short = 'o', long, value_name = "FILE", help = "write IR to file instead of stdout")]
        output: Option<PathBuf>,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Some(Command::Ir { expr, optimize, output }) => {
            let ast = parse(&expr);
            let context = Context::create();
            let cg = codegen::CodeGen::new(&context);
            cg.compile(&ast);
            if optimize {
                cg.optimize();
            }
            match output {
                Some(path) => cg.write_ir(&path),
                None => cg.print_ir(),
            }
        }
        None => {
            let expr = args.expr.unwrap_or_else(|| {
                eprintln!("error: expression required");
                std::process::exit(1);
            });
            let ast = parse(&expr);
            let context = Context::create();
            let cg = codegen::CodeGen::new(&context);
            cg.compile(&ast);
            if args.optimize {
                cg.optimize();
            }
            cg.jit_run();
        }
    }
}

fn parse(input: &str) -> ast::Expr {
    parser::parse(input).unwrap_or_else(|e| {
        eprintln!("parse error: {e}");
        std::process::exit(1);
    })
}
