use std::collections::HashMap;

use inkwell::AddressSpace;
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::values::{FloatValue, PointerValue};

use crate::ast::{BinOp, Expr, Program};

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            module: context.create_module("calc"),
            builder: context.create_builder(),
        }
    }

    // Emit:
    //   @fmt = private constant [N x i8] c"%g\n\00"
    //   declare i32 @printf(ptr, ...)
    //   define i32 @main() {
    //     %x = alloca double; store <val>, %x   <- one per binding
    //     ...
    //     %result = load / compute body
    //     call printf(fmt, %result)
    //     ret i32 0
    //   }
    pub fn compile(&self, program: &Program) -> Result<(), String> {
        let i32_type = self.context.i32_type();
        let f64_type = self.context.f64_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());

        let printf_type = i32_type.fn_type(&[ptr_type.into()], true);
        let printf = self.module.add_function("printf", printf_type, None);

        let fmt_str = self.context.const_string(b"%g\n", true);
        let fmt_global = self.module.add_global(fmt_str.get_type(), None, "fmt");
        fmt_global.set_initializer(&fmt_str);
        fmt_global.set_linkage(Linkage::Private);
        fmt_global.set_constant(true);

        let main_fn = self.module.add_function("main", i32_type.fn_type(&[], false), None);
        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);

        let mut vars: HashMap<String, PointerValue<'ctx>> = HashMap::new();
        for (name, expr) in &program.bindings {
            let val = self.compile_expr(expr, &vars)?;
            let ptr = self.builder.build_alloca(f64_type, name).unwrap();
            self.builder.build_store(ptr, val).unwrap();
            vars.insert(name.clone(), ptr);
        }

        let result = self.compile_expr(&program.body, &vars)?;

        self.builder
            .build_call(printf, &[fmt_global.as_pointer_value().into(), result.into()], "")
            .unwrap();

        self.builder
            .build_return(Some(&i32_type.const_int(0, false)))
            .unwrap();

        Ok(())
    }

    fn compile_expr(&self, expr: &Expr, vars: &HashMap<String, PointerValue<'ctx>>) -> Result<FloatValue<'ctx>, String> {
        match expr {
            Expr::Number(n) => Ok(self.context.f64_type().const_float(*n)),
            Expr::Var(name) => {
                let ptr = vars.get(name)
                    .ok_or_else(|| format!("undefined variable: {name}"))?;
                Ok(self.builder
                    .build_load(self.context.f64_type(), *ptr, name)
                    .unwrap()
                    .into_float_value())
            }
            Expr::Neg(inner) => {
                let val = self.compile_expr(inner, vars)?;
                Ok(self.builder.build_float_neg(val, "neg").unwrap())
            }
            Expr::BinOp { op, left, right } => {
                let l = self.compile_expr(left, vars)?;
                let r = self.compile_expr(right, vars)?;
                Ok(match op {
                    BinOp::Add => self.builder.build_float_add(l, r, "add").unwrap(),
                    BinOp::Sub => self.builder.build_float_sub(l, r, "sub").unwrap(),
                    BinOp::Mul => self.builder.build_float_mul(l, r, "mul").unwrap(),
                    BinOp::Div => self.builder.build_float_div(l, r, "div").unwrap(),
                })
            }
        }
    }

    pub fn optimize(&self) {
        Target::initialize_native(&InitializationConfig::default()).unwrap();
        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).unwrap();
        let machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .unwrap();
        // mem2reg promotes alloca/store/load to SSA registers before the rest run
        self.module
            .run_passes("mem2reg,instcombine,reassociate,gvn,simplifycfg", &machine, PassBuilderOptions::create())
            .unwrap();
    }

    pub fn print_ir(&self) {
        println!("{}", self.module.print_to_string().to_string());
    }

    pub fn write_ir(&self, path: &std::path::Path) {
        self.module.print_to_file(path).unwrap_or_else(|e| {
            eprintln!("error writing IR: {}", e.to_string());
            std::process::exit(1);
        });
    }

    // Consumes self because create_jit_execution_engine takes ownership of the module.
    pub fn jit_run(self) {
        let ee = self.module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();
        unsafe {
            ee.get_function::<unsafe extern "C" fn() -> i32>("main")
                .unwrap()
                .call();
        }
    }
}
