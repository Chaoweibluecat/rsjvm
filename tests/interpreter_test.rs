//! 解释器集成测试
//!
//! 运行: cargo test

use rsjvm::interpreter::Interpreter;
use rsjvm::runtime::frame::JvmValue;

#[test]
fn test_iconst_and_ireturn() {
    // 测试: iconst_1; ireturn
    let bytecode = vec![0x04, 0xac];
    let mut interpreter = Interpreter::new();

    // 应该能成功执行并返回1
    match interpreter.execute_method(&bytecode, 0, 1) {
        Ok(Some(JvmValue::Int(1))) => (),
        Ok(other) => panic!("期望返回Int(1), 实际: {:?}", other),
        Err(e) => panic!("执行失败: {:?}", e),
    }
}

#[test]
fn test_simple_add() {
    // 测试: iconst_1; iconst_2; iadd; ireturn (应该返回3)
    let bytecode = vec![
        0x04, // iconst_1
        0x05, // iconst_2
        0x60, // iadd
        0xac, // ireturn
    ];

    let mut interpreter = Interpreter::new();
    match interpreter.execute_method(&bytecode, 0, 2) {
        Ok(Some(JvmValue::Int(3))) => (),
        result => panic!("期望返回Int(3), 实际: {:?}", result),
    }
}

#[test]
fn test_local_variables() {
    // 测试局部变量: iconst_5; istore_0; iload_0; ireturn (应该返回5)
    let bytecode = vec![
        0x08, // iconst_5
        0x3b, // istore_0
        0x1a, // iload_0
        0xac, // ireturn
    ];

    let mut interpreter = Interpreter::new();
    match interpreter.execute_method(&bytecode, 1, 1) {
        Ok(Some(JvmValue::Int(5))) => (),
        result => panic!("期望返回Int(5), 实际: {:?}", result),
    }
}

#[test]
fn test_bipush() {
    // 测试: bipush 42; ireturn (应该返回42)
    let bytecode = vec![
        0x10, 42, // bipush 42
        0xac,     // ireturn
    ];

    let mut interpreter = Interpreter::new();
    match interpreter.execute_method(&bytecode, 0, 1) {
        Ok(Some(JvmValue::Int(42))) => (),
        result => panic!("期望返回Int(42), 实际: {:?}", result),
    }
}

#[test]
fn test_sipush() {
    // 测试: sipush 1000; ireturn (应该返回1000)
    let bytecode = vec![
        0x11, 0x03, 0xe8, // sipush 1000
        0xac,             // ireturn
    ];

    let mut interpreter = Interpreter::new();
    match interpreter.execute_method(&bytecode, 0, 1) {
        Ok(Some(JvmValue::Int(1000))) => (),
        result => panic!("期望返回Int(1000), 实际: {:?}", result),
    }
}

#[test]
fn test_arithmetic() {
    // 测试四则运算
    let test_cases = vec![
        (vec![0x04, 0x05, 0x60, 0xac], 3, "add"),    // 1 + 2 = 3
        (vec![0x08, 0x05, 0x64, 0xac], 3, "sub"),    // 5 - 2 = 3
        (vec![0x06, 0x07, 0x68, 0xac], 12, "mul"),   // 3 * 4 = 12
        (vec![0x08, 0x05, 0x6c, 0xac], 2, "div"),    // 5 / 2 = 2
    ];

    for (bytecode, expected, name) in test_cases {
        let mut interpreter = Interpreter::new();
        match interpreter.execute_method(&bytecode, 0, 2) {
            Ok(Some(JvmValue::Int(val))) if val == expected => (),
            result => panic!("{} 失败: 期望 {}, 实际 {:?}", name, expected, result),
        }
    }
}

#[test]
#[should_panic(expected = "Division by zero")]
fn test_divide_by_zero() {
    // 测试除以零
    let bytecode = vec![
        0x04, // iconst_1
        0x03, // iconst_0
        0x6c, // idiv (应该panic)
    ];

    let mut interpreter = Interpreter::new();
    interpreter.execute_method(&bytecode, 0, 2).unwrap();
}

#[test]
fn test_frame_operations() {
    use rsjvm::runtime::Frame;

    let mut frame = Frame::new(5, 10);

    // 测试压栈和弹栈
    frame.push(JvmValue::Int(42));
    assert_eq!(frame.stack_size(), 1);

    let val = frame.pop_int().unwrap();
    assert_eq!(val, 42);
    assert_eq!(frame.stack_size(), 0);

    // 测试局部变量
    frame.set_local(0, JvmValue::Int(100)).unwrap();
    match frame.get_local(0).unwrap() {
        JvmValue::Int(v) => assert_eq!(v, &100),
        _ => panic!("Expected Int"),
    }
}
