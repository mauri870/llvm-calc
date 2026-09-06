use inkwell::OptimizationLevel;
use inkwell::context::Context;

fn main() {
    let context = Context::create();
    let module = context.create_module("calc");
    let builder = context.create_builder();

    // emit double @eval() { ret double 42.0 }
    let f64_type = context.f64_type();
    let fn_type = f64_type.fn_type(&[], false);
    let function = module.add_function("eval", fn_type, None);
    let entry = context.append_basic_block(function, "entry");
    builder.position_at_end(entry);
    let value = f64_type.const_float(42.0);
    builder.build_return(Some(&value)).unwrap();

    println!("LLVM IR");
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
