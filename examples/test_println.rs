use anyhow::Result;
use rsjvm::classfile::ClassFile;
use rsjvm::interpreter::Interpreter;

fn main() -> Result<()> {
    println!("=== 测试 println 支持 ===\n");

    // 1. 创建解释器
    let mut interpreter = Interpreter::new();

    // 2. 加载 HelloPrintln 类
    let class_file = ClassFile::from_file("examples/HelloPrintln.class")?;
    let class_name = interpreter.load_class(class_file)?;
    println!("✓ 类已加载: {}\n", class_name);

    // 3. 获取 main 方法信息（克隆以避免借用冲突）
    let (code, max_locals, max_stack) = {
        let class_meta = interpreter.metaspace.get_class(&class_name)?;
        let main_method = class_meta.find_method("main", "([Ljava/lang/String;)V")?;
        (main_method.code.clone(), main_method.max_locals, main_method.max_stack)
    };

    // 4. 执行 main 方法
    println!("执行 main 方法:\n");
    println!("--- 程序输出开始 ---");
    let result = interpreter.execute_method_with_class(
        &class_name,
        &code,
        max_locals,
        max_stack,
    )?;
    println!("--- 程序输出结束 ---\n");

    println!("✓ main 方法执行完成，返回值: {:?}", result);
    println!("\n🎉 println 测试成功！");

    Ok(())
}
