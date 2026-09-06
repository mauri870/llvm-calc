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
    #[arg(allow_hyphen_values = true)]
    expr: Option<String>,
    #[arg(short = 'O', long, help = "run optimization passes")]
    optimize: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Print LLVM IR and exit
    Ir {
        #[arg(allow_hyphen_values = true)]
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
            cg.compile(&ast).unwrap_or_else(|e| die(&e));
            if optimize {
                cg.optimize();
            }
            match output {
                Some(path) => cg.write_ir(&path),
                None => cg.print_ir(),
            }
        }
        None => match args.expr {
            Some(expr) => {
                let ast = parse(&expr);
                let context = Context::create();
                let cg = codegen::CodeGen::new(&context);
                cg.compile(&ast).unwrap_or_else(|e| die(&e));
                if args.optimize {
                    cg.optimize();
                }
                cg.jit_run();
            }
            None => repl(),
        },
    }
}

fn repl() {
    use std::io::{self, BufRead, Write};

    let context = Context::create();
    let mut stdout = io::stdout();
    let stdin = io::stdin();

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("error: {e}");
                break;
            }
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        match parser::parse(input) {
            Ok(ast) => {
                let cg = codegen::CodeGen::new(&context);
                match cg.compile(&ast) {
                    Ok(()) => cg.jit_run(),
                    Err(e) => eprintln!("error: {e}"),
                }
            }
            Err(e) => eprintln!("parse error: {e}"),
        }
    }
}

fn parse(input: &str) -> ast::Program {
    parser::parse(input).unwrap_or_else(|e| die(&format!("parse error: {e}")))
}

fn die(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(1);
}
