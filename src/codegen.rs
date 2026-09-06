use std::collections::HashMap;

use inkwell::AddressSpace;
use inkwell::FloatPredicate;
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::values::{FloatValue, FunctionValue, PointerValue};

use crate::ast::{BinOp, Block, CmpOp, Cond, Expr, Program};

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
    //     %result = <body expr>
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
            let val = self.compile_expr(expr, &mut vars, main_fn)?;
            let ptr = self.builder.build_alloca(f64_type, name).unwrap();
            self.builder.build_store(ptr, val).unwrap();
            vars.insert(name.clone(), ptr);
        }

        let result = self.compile_expr(&program.body, &mut vars, main_fn)?;

        self.builder
            .build_call(printf, &[fmt_global.as_pointer_value().into(), result.into()], "")
            .unwrap();

        self.builder
            .build_return(Some(&i32_type.const_int(0, false)))
            .unwrap();

        Ok(())
    }

    // Compile a block (zero or more assignments, then a result expression).
    // Rebinding an already-defined variable stores to its existing alloca
    // rather than creating a new one — this is how mutation works in loops.
    fn compile_block(
        &self,
        block: &Block,
        vars: &mut HashMap<String, PointerValue<'ctx>>,
        function: FunctionValue<'ctx>,
    ) -> Result<FloatValue<'ctx>, String> {
        let f64_type = self.context.f64_type();
        for (name, expr) in &block.bindings {
            let val = self.compile_expr(expr, vars, function)?;
            if let Some(&ptr) = vars.get(name.as_str()) {
                self.builder.build_store(ptr, val).unwrap();
            } else {
                let ptr = self.builder.build_alloca(f64_type, name).unwrap();
                self.builder.build_store(ptr, val).unwrap();
                vars.insert(name.clone(), ptr);
            }
        }
        self.compile_expr(&block.body, vars, function)
    }

    fn compile_expr(
        &self,
        expr: &Expr,
        vars: &mut HashMap<String, PointerValue<'ctx>>,
        function: FunctionValue<'ctx>,
    ) -> Result<FloatValue<'ctx>, String> {
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
                let val = self.compile_expr(inner, vars, function)?;
                Ok(self.builder.build_float_neg(val, "neg").unwrap())
            }
            Expr::BinOp { op, left, right } => {
                let l = self.compile_expr(left, vars, function)?;
                let r = self.compile_expr(right, vars, function)?;
                Ok(match op {
                    BinOp::Add => self.builder.build_float_add(l, r, "add").unwrap(),
                    BinOp::Sub => self.builder.build_float_sub(l, r, "sub").unwrap(),
                    BinOp::Mul => self.builder.build_float_mul(l, r, "mul").unwrap(),
                    BinOp::Div => self.builder.build_float_div(l, r, "div").unwrap(),
                })
            }
            Expr::If { cond, then, else_ } => {
                self.compile_if(cond, then, else_, vars, function)
            }
            Expr::While { cond, body } => {
                self.compile_while(cond, body, vars, function)
            }
        }
    }

    // Standard if-then-else with a phi node at the merge block:
    //
    //   %cond = fcmp o<op> %l, %r
    //   br i1 %cond, label %then, label %else
    // then:
    //   %then_val = <then expr>
    //   br label %merge
    // else:
    //   %else_val = <else expr>
    //   br label %merge
    // merge:
    //   %result = phi double [ %then_val, %then ], [ %else_val, %else ]
    fn compile_if(
        &self,
        cond: &Cond,
        then: &Expr,
        else_: &Expr,
        vars: &mut HashMap<String, PointerValue<'ctx>>,
        function: FunctionValue<'ctx>,
    ) -> Result<FloatValue<'ctx>, String> {
        let l = self.compile_expr(&cond.left, vars, function)?;
        let r = self.compile_expr(&cond.right, vars, function)?;
        let cond_val = self.builder
            .build_float_compare(float_predicate(&cond.op), l, r, "cond")
            .unwrap();

        let then_block  = self.context.append_basic_block(function, "then");
        let else_block  = self.context.append_basic_block(function, "else");
        let merge_block = self.context.append_basic_block(function, "merge");

        self.builder.build_conditional_branch(cond_val, then_block, else_block).unwrap();

        self.builder.position_at_end(then_block);
        let then_val = self.compile_expr(then, vars, function)?;
        self.builder.build_unconditional_branch(merge_block).unwrap();
        let then_exit = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(else_block);
        let else_val = self.compile_expr(else_, vars, function)?;
        self.builder.build_unconditional_branch(merge_block).unwrap();
        let else_exit = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(merge_block);
        let phi = self.builder.build_phi(self.context.f64_type(), "result").unwrap();
        phi.add_incoming(&[(&then_val, then_exit), (&else_val, else_exit)]);

        Ok(phi.as_basic_value().into_float_value())
    }

    // While loop using an alloca for the result (0.0 if body never ran):
    //
    //   %result_ptr = alloca double
    //   store double 0.0, ptr %result_ptr
    //   br label %loop_header
    // loop_header:
    //   %cond = fcmp o<op> %l, %r
    //   br i1 %cond, label %loop_body, label %loop_exit
    // loop_body:
    //   <block — rebinds via existing allocas for mutation>
    //   store double %body_val, ptr %result_ptr
    //   br label %loop_header
    // loop_exit:
    //   %result = load double, ptr %result_ptr
    //
    // Because variable state lives in allocas (not SSA registers), mutation
    // across iterations is just store/load on the same alloca — no phi nodes
    // needed for loop variables. mem2reg cleans this up when -O is used.
    fn compile_while(
        &self,
        cond: &Cond,
        body: &Block,
        vars: &mut HashMap<String, PointerValue<'ctx>>,
        function: FunctionValue<'ctx>,
    ) -> Result<FloatValue<'ctx>, String> {
        let f64_type = self.context.f64_type();

        let result_ptr = self.builder.build_alloca(f64_type, "while_result").unwrap();
        self.builder.build_store(result_ptr, f64_type.const_float(0.0)).unwrap();

        let loop_header = self.context.append_basic_block(function, "loop_header");
        let loop_body   = self.context.append_basic_block(function, "loop_body");
        let loop_exit   = self.context.append_basic_block(function, "loop_exit");

        self.builder.build_unconditional_branch(loop_header).unwrap();

        // Evaluate condition and branch.
        self.builder.position_at_end(loop_header);
        let l = self.compile_expr(&cond.left, vars, function)?;
        let r = self.compile_expr(&cond.right, vars, function)?;
        let cond_val = self.builder
            .build_float_compare(float_predicate(&cond.op), l, r, "while_cond")
            .unwrap();
        self.builder.build_conditional_branch(cond_val, loop_body, loop_exit).unwrap();

        // Compile body; store its result; branch back.
        self.builder.position_at_end(loop_body);
        let body_val = self.compile_block(body, vars, function)?;
        self.builder.build_store(result_ptr, body_val).unwrap();
        self.builder.build_unconditional_branch(loop_header).unwrap();

        // Load and return the last body value.
        self.builder.position_at_end(loop_exit);
        Ok(self.builder
            .build_load(f64_type, result_ptr, "while_result")
            .unwrap()
            .into_float_value())
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

fn float_predicate(op: &CmpOp) -> FloatPredicate {
    match op {
        CmpOp::Lt => FloatPredicate::OLT,
        CmpOp::Gt => FloatPredicate::OGT,
        CmpOp::Eq => FloatPredicate::OEQ,
        CmpOp::Ne => FloatPredicate::ONE,
        CmpOp::Le => FloatPredicate::OLE,
        CmpOp::Ge => FloatPredicate::OGE,
    }
}
