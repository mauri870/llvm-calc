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
    #[arg(
        short = 'O',
        value_name = "LEVEL",
        value_parser = clap::value_parser!(u8).range(0..=3),
        help = "optimization level: -O0, -O1, -O2, -O3",
    )]
    optimize: Option<u8>,
}

#[derive(Subcommand)]
enum Command {
    /// Print LLVM IR and exit
    Ir {
        #[arg(allow_hyphen_values = true)]
        expr: String,
        #[arg(
            short = 'O',
            value_name = "LEVEL",
            value_parser = clap::value_parser!(u8).range(0..=3),
            help = "optimization level: -O0, -O1, -O2, -O3",
        )]
        optimize: Option<u8>,
        #[arg(short = 'o', long, value_name = "FILE", help = "write IR to file instead of stdout")]
        output: Option<PathBuf>,
    },
}

// Split -O<n> into two tokens before clap sees them.
// allow_hyphen_values on the positional expr would otherwise consume -O2 as
// the expression value rather than recognizing it as the -O flag.
fn preprocess_args(raw: impl Iterator<Item = String>) -> Vec<String> {
    raw.flat_map(|arg| {
        if let Some(suffix) = arg.strip_prefix("-O") {
            if suffix.len() == 1 && suffix.as_bytes()[0].is_ascii_digit() {
                return vec!["-O".to_string(), suffix.to_string()];
            }
        }
        vec![arg]
    })
    .collect()
}

fn main() {
    let args = Args::parse_from(preprocess_args(std::env::args()));

    match args.command {
        Some(Command::Ir { expr, optimize, output }) => {
            let ast = parse(&expr);
            let context = Context::create();
            let cg = codegen::CodeGen::new(&context);
            cg.compile(&ast).unwrap_or_else(|e| die(&e));
            if let Some(level) = optimize {
                cg.optimize(level);
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
                if let Some(level) = args.optimize {
                    cg.optimize(level);
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
