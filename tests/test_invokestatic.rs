//! 测试 invokestatic 指令

use rsjvm::classfile::ClassFile;
use rsjvm::interpreter::Interpreter;
use rsjvm::runtime::frame::JvmValue;
use rsjvm::Result;

#[test]
fn test_invokestatic_simple() -> Result<()> {
    // 1. 创建解释器
    let mut interpreter = Interpreter::new();

    // 2. 加载 TestInvokeStatic 类
    let class_file = ClassFile::from_file("examples/TestInvokeStatic.class")?;
    let class_name = interpreter.load_class(class_file)?;

    // 3. 获取 main 方法（克隆数据以避免借用冲突）
    let (code, max_locals, max_stack) = {
        let class_meta = interpreter.metaspace.get_class(&class_name)?;
        let main_method = class_meta.find_method("main", "([Ljava/lang/String;)V")?;
        (main_method.code.clone(), main_method.max_locals, main_method.max_stack)
    };

    // 4. 执行 main 方法（会调用 sum_a_and_b）
    let result = interpreter.execute_method_with_class(
        &class_name,
        &code,
        max_locals,
        max_stack,
    )?;

    // main 方法是 void，应该没有返回值
    assert!(result.is_none());

    Ok(())
}

#[test]
fn test_invokestatic_with_return_value() -> Result<()> {
    // 1. 创建解释器
    let mut interpreter = Interpreter::new();

    // 2. 加载类
    let class_file = ClassFile::from_file("examples/TestInvokeStatic.class")?;
    let class_name = interpreter.load_class(class_file)?;

    // 3. 获取方法信息（克隆以避免借用冲突）
    let (code, max_locals, max_stack) = {
        let class_meta = interpreter.metaspace.get_class(&class_name)?;
        let method = class_meta.find_method("sum_a_and_b", "(II)I")?;
        (method.code.clone(), method.max_locals, method.max_stack)
    };

    // 4. 创建栈帧并设置参数
    let mut frame = rsjvm::runtime::Frame::new(max_locals, max_stack);
    frame.set_local(0, JvmValue::Int(10))?;
    frame.set_local(1, JvmValue::Int(20))?;

    // 5. 执行方法
    let result = interpreter.execute_method_in_frame(&code, &mut frame, &class_name)?;

    // 6. 验证结果
    assert!(result.is_some());
    if let Some(JvmValue::Int(val)) = result {
        assert_eq!(val, 30);
    } else {
        panic!("Expected Int return value");
    }

    Ok(())
}

#[test]
fn test_invokestatic_multiple_calls() -> Result<()> {
    // 测试多次调用同一个方法
    let mut interpreter = Interpreter::new();

    let class_file = ClassFile::from_file("examples/TestInvokeStatic.class")?;
    let class_name = interpreter.load_class(class_file)?;

    // 获取方法信息
    let (code, max_locals, max_stack) = {
        let class_meta = interpreter.metaspace.get_class(&class_name)?;
        let method = class_meta.find_method("sum_a_and_b", "(II)I")?;
        (method.code.clone(), method.max_locals, method.max_stack)
    };

    // 第一次调用
    let mut frame1 = rsjvm::runtime::Frame::new(max_locals, max_stack);
    frame1.set_local(0, JvmValue::Int(1))?;
    frame1.set_local(1, JvmValue::Int(2))?;
    let result1 = interpreter.execute_method_in_frame(&code, &mut frame1, &class_name)?;

    // 第二次调用
    let mut frame2 = rsjvm::runtime::Frame::new(max_locals, max_stack);
    frame2.set_local(0, JvmValue::Int(100))?;
    frame2.set_local(1, JvmValue::Int(200))?;
    let result2 = interpreter.execute_method_in_frame(&code, &mut frame2, &class_name)?;

    // 验证结果
    if let (Some(JvmValue::Int(v1)), Some(JvmValue::Int(v2))) = (result1, result2) {
        assert_eq!(v1, 3);
        assert_eq!(v2, 300);
    } else {
        panic!("Expected Int return values");
    }

    Ok(())
}
