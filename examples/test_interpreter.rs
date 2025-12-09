//! 手动测试解释器
//!
//! 这个文件展示如何手动构造字节码并用解释器执行
//!
//! 运行方式：
//! ```bash
//! rustc --edition 2021 examples/test_interpreter.rs -L target/release/deps --extern rsjvm=target/release/librsjvm.rlib
//! ./test_interpreter
//! ```
//!
//! 或者作为集成测试放到 tests/ 目录

use rsjvm::interpreter::Interpreter;

fn main() {
    println!("=== 测试1: returnOne() ===");
    test_return_one();

    println!("\n=== 测试2: addOne() ===");
    test_add_one();

    println!("\n=== 测试3: calculate() ===");
    test_calculate();
}

/// 测试 returnOne() 方法
/// Java: public static int returnOne() { return 1; }
/// 字节码: 04 ac
fn test_return_one() {
    let bytecode: Vec<u8> = vec![
        0x04, // iconst_1
        0xac, // ireturn
    ];

    let mut interpreter = Interpreter::new();

    println!("字节码: {:02x?}", bytecode);
    println!("执行中...");

    match interpreter.execute_method(&bytecode, 0, 1) {
        Ok(_) => println!("✓ 执行成功！"),
        Err(e) => println!("✗ 执行失败: {}", e),
    }
}

/// 测试 addOne() 方法
/// Java:
/// public static int addOne() {
///     int a = 1;
///     int b = 0;
///     return a + b;
/// }
/// 字节码: 04 3b 03 3c 1a 1b 60 ac
fn test_add_one() {
    let bytecode: Vec<u8> = vec![
        0x04, // iconst_1      -> stack: [1]
        0x3b, // istore_0      -> locals[0]=1, stack: []
        0x03, // iconst_0      -> stack: [0]
        0x3c, // istore_1      -> locals[1]=0, stack: []
        0x1a, // iload_0       -> stack: [1]
        0x1b, // iload_1       -> stack: [1, 0]
        0x60, // iadd          -> stack: [1]
        0xac, // ireturn
    ];

    let mut interpreter = Interpreter::new();

    println!("字节码: {:02x?}", bytecode);
    println!("执行中...");

    match interpreter.execute_method(&bytecode, 2, 2) {
        Ok(_) => println!("✓ 执行成功！结果应该是1"),
        Err(e) => println!("✗ 执行失败: {}", e),
    }
}

/// 测试 calculate() 方法
/// Java:
/// public static int calculate() {
///     int a = 10;
///     int b = 20;
///     int c = a + b;
///     return c;
/// }
/// 字节码: 10 0a 3b 10 14 3c 1a 1b 60 3d 1c ac
fn test_calculate() {
    let bytecode: Vec<u8> = vec![
        0x10, 0x0a, // bipush 10    -> stack: [10]
        0x3b,       // istore_0     -> locals[0]=10, stack: []
        0x10, 0x14, // bipush 20    -> stack: [20]
        0x3c,       // istore_1     -> locals[1]=20, stack: []
        0x1a,       // iload_0      -> stack: [10]
        0x1b,       // iload_1      -> stack: [10, 20]
        0x60,       // iadd         -> stack: [30]
        0x3d,       // istore_2     -> locals[2]=30, stack: []
        0x1c,       // iload_2      -> stack: [30]
        0xac,       // ireturn
    ];

    let mut interpreter = Interpreter::new();

    println!("字节码: {:02x?}", bytecode);
    println!("执行中...");

    match interpreter.execute_method(&bytecode, 3, 2) {
        Ok(_) => println!("✓ 执行成功！结果应该是30"),
        Err(e) => println!("✗ 执行失败: {}", e),
    }
}

/// 测试错误情况：除以零
fn _test_divide_by_zero() {
    let bytecode: Vec<u8> = vec![
        0x04, // iconst_1
        0x03, // iconst_0
        0x6c, // idiv (应该报错)
    ];

    let mut interpreter = Interpreter::new();

    match interpreter.execute_method(&bytecode, 0, 2) {
        Ok(_) => println!("✗ 应该报错但成功了！"),
        Err(e) => println!("✓ 正确捕获错误: {}", e),
    }
}
