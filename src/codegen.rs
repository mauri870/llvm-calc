use inkwell::AddressSpace;
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::values::FloatValue;

use crate::ast::{BinOp, Expr};

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

    // Emit a full mini LLVM IR program:
    //   @fmt = private constant [N x i8] c"%g\n\00"
    //   declare i32 @printf(ptr, ...)
    //   define i32 @main() { ...; call printf(fmt, <expr>); ret i32 0 }
    pub fn compile(&self, expr: &Expr) {
        let i32_type = self.context.i32_type();
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

        let result = self.compile_expr(expr);

        self.builder
            .build_call(printf, &[fmt_global.as_pointer_value().into(), result.into()], "")
            .unwrap();

        self.builder
            .build_return(Some(&i32_type.const_int(0, false)))
            .unwrap();
    }

    fn compile_expr(&self, expr: &Expr) -> FloatValue<'ctx> {
        match expr {
            Expr::Number(n) => self.context.f64_type().const_float(*n),
            Expr::BinOp { op, left, right } => {
                let l = self.compile_expr(left);
                let r = self.compile_expr(right);
                match op {
                    BinOp::Add => self.builder.build_float_add(l, r, "add").unwrap(),
                    BinOp::Sub => self.builder.build_float_sub(l, r, "sub").unwrap(),
                    BinOp::Mul => self.builder.build_float_mul(l, r, "mul").unwrap(),
                    BinOp::Div => self.builder.build_float_div(l, r, "div").unwrap(),
                }
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
        self.module
            .run_passes("instcombine,reassociate,gvn,simplifycfg", &machine, PassBuilderOptions::create())
            .unwrap();
    }

    pub fn print_ir(&self) {
        println!("{}", self.module.print_to_string().to_string());
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
