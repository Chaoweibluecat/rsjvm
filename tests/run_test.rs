//! 测试完整的 run 流程
//!
//! 这个测试模拟完整的加载class文件 -> 解析 -> 执行的流程
//! 运行: cargo test --test run_test -- --nocapture

use rsjvm::classfile::ClassFile;
use rsjvm::interpreter::Interpreter;
use rsjvm::runtime::frame::JvmValue;
use std::path::PathBuf;

#[test]
fn test_run_return_one() {
    println!("\n========== 测试 returnOne() ==========");

    // 1. 加载class文件
    let path = PathBuf::from("examples/ReturnOne.class");
    println!("📂 加载文件: {:?}", path);

    let class_file = ClassFile::from_file(&path).expect("Failed to load class file");
    println!("✓ 成功加载class文件");

    // 2. 获取类名
    let class_name = class_file
        .get_class_name()
        .expect("Failed to get class name");
    println!("📝 类名: {}", class_name);
    // println!("🔍 cp: {:?}", class_file.constant_pool);
    class_file.constant_pool.debug_print();
    // 3. 查找方法
    let method_name = "returnOne";
    println!("🔍 查找方法: {}", method_name);

    let mut found_method = None;
    for method in &class_file.methods {
        let name = class_file
            .constant_pool
            .get_utf8(method.name_index)
            .unwrap();
        if name == method_name {
            found_method = Some(method);
            break;
        }
    }

    let method = found_method.expect("Method not found");
    println!("✓ 找到方法");

    // 4. 获取方法签名
    let descriptor = class_file
        .constant_pool
        .get_utf8(method.descriptor_index)
        .unwrap();
    println!("📋 方法签名: {} : {}", method_name, descriptor);

    // 5. 查找Code属性
    println!("🔍 查找Code属性...");
    let mut code_attr = None;
    for attr in &method.attributes {
        let attr_name = class_file.constant_pool.get_utf8(attr.name_index).unwrap();
        println!("  - 属性: {}", attr_name);
        if attr_name == "Code" {
            code_attr = Some(attr.parse_code_attribute().expect("Failed to parse code"));
            break;
        }
    }

    let code = code_attr.expect("No Code attribute");
    println!("✓ 找到Code属性");

    // 6. 显示方法信息
    println!("\n=== 方法信息 ===");
    println!("max_stack: {}", code.max_stack);
    println!("max_locals: {}", code.max_locals);
    println!("code_length: {}", code.code.len());
    println!("字节码: {:02x?}", code.code);

    // 7. 执行方法
    println!("\n=== 开始执行 ===");
    let mut interpreter = Interpreter::new();

    match interpreter.execute_method(
        &code.code,
        code.max_locals as usize,
        code.max_stack as usize,
    ) {
        Ok(return_value) => {
            println!("✓ 执行成功！");

            if let Some(JvmValue::Int(val)) = return_value {
                println!("📤 返回值: {}", val);
                assert_eq!(val, 1, "期望返回1");
                println!("✅ 断言通过！");
            } else {
                panic!("期望返回Int(1), 实际: {:?}", return_value);
            }
        }
        Err(e) => {
            println!("✗ 执行失败: {}", e);
            panic!("Execution failed: {}", e);
        }
    }
}

#[test]
fn test_run_add_one() {
    println!("\n========== 测试 addOne() ==========");

    let path = PathBuf::from("examples/ReturnOne.class");
    let class_file = ClassFile::from_file(&path).expect("Failed to load class file");

    // 查找 addOne 方法
    let method_name = "addOne";
    println!("🔍 查找方法: {}", method_name);

    let method = class_file
        .methods
        .iter()
        .find(|m| class_file.constant_pool.get_utf8(m.name_index).unwrap() == method_name)
        .expect("Method not found");

    let descriptor = class_file
        .constant_pool
        .get_utf8(method.descriptor_index)
        .unwrap();
    println!("📋 方法签名: {} : {}", method_name, descriptor);

    // 获取Code属性
    let code = method
        .attributes
        .iter()
        .find(|attr| class_file.constant_pool.get_utf8(attr.name_index).unwrap() == "Code")
        .expect("No Code attribute")
        .parse_code_attribute()
        .expect("Failed to parse code");

    println!("\n=== 方法信息 ===");
    println!("max_stack: {}", code.max_stack);
    println!("max_locals: {}", code.max_locals);
    println!("字节码: {:02x?}", code.code);

    // 详细解析字节码
    println!("\n=== 字节码分析 ===");
    println!("0x04 = iconst_1    // 压入常量1");
    println!("0x3b = istore_0    // 存入局部变量0");
    println!("0x03 = iconst_0    // 压入常量0");
    println!("0x3c = istore_1    // 存入局部变量1");
    println!("0x1a = iload_0     // 加载局部变量0");
    println!("0x1b = iload_1     // 加载局部变量1");
    println!("0x60 = iadd        // 整数加法");
    println!("0xac = ireturn     // 返回整数");

    // 执行
    println!("\n=== 开始执行 ===");
    let mut interpreter = Interpreter::new();

    match interpreter.execute_method(
        &code.code,
        code.max_locals as usize,
        code.max_stack as usize,
    ) {
        Ok(Some(JvmValue::Int(val))) => {
            println!("✓ 执行成功！");
            println!("📤 返回值: {} (期望: 1)", val);
            assert_eq!(val, 1, "1 + 0 应该等于 1");
            println!("✅ 断言通过！");
        }
        result => panic!("期望返回Int(1), 实际: {:?}", result),
    }
}

#[test]
fn test_run_calculate() {
    println!("\n========== 测试 calculate() ==========");

    let path = PathBuf::from("examples/ReturnOne.class");
    let class_file = ClassFile::from_file(&path).expect("Failed to load class file");

    // 查找 calculate 方法
    let method_name = "calculate";
    println!("🔍 查找方法: {}", method_name);

    let method = class_file
        .methods
        .iter()
        .find(|m| class_file.constant_pool.get_utf8(m.name_index).unwrap() == method_name)
        .expect("Method not found");

    let descriptor = class_file
        .constant_pool
        .get_utf8(method.descriptor_index)
        .unwrap();
    println!("📋 方法签名: {} : {}", method_name, descriptor);

    // 获取Code属性
    let code = method
        .attributes
        .iter()
        .find(|attr| class_file.constant_pool.get_utf8(attr.name_index).unwrap() == "Code")
        .expect("No Code attribute")
        .parse_code_attribute()
        .expect("Failed to parse code");

    println!("\n=== 方法信息 ===");
    println!("max_stack: {}", code.max_stack);
    println!("max_locals: {}", code.max_locals);
    println!("字节码: {:02x?}", code.code);

    // 详细解析字节码
    println!("\n=== 字节码分析 ===");
    println!("0x10 0x0a = bipush 10   // 压入常量10");
    println!("0x3b      = istore_0    // 存入局部变量0 (a=10)");
    println!("0x10 0x14 = bipush 20   // 压入常量20");
    println!("0x3c      = istore_1    // 存入局部变量1 (b=20)");
    println!("0x1a      = iload_0     // 加载局部变量0 (a)");
    println!("0x1b      = iload_1     // 加载局部变量1 (b)");
    println!("0x60      = iadd        // 整数加法 (a+b)");
    println!("0x3d      = istore_2    // 存入局部变量2 (c=30)");
    println!("0x1c      = iload_2     // 加载局部变量2 (c)");
    println!("0xac      = ireturn     // 返回整数");

    // 执行
    println!("\n=== 开始执行 ===");
    let mut interpreter = Interpreter::new();

    match interpreter.execute_method(
        &code.code,
        code.max_locals as usize,
        code.max_stack as usize,
    ) {
        Ok(Some(JvmValue::Int(val))) => {
            println!("✓ 执行成功！");
            println!("📤 返回值: {} (期望: 30)", val);
            assert_eq!(val, 30, "10 + 20 应该等于 30");
            println!("✅ 断言通过！");
        }
        result => panic!("期望返回Int(30), 实际: {:?}", result),
    }
}

#[test]
fn test_all_methods_in_return_one() {
    println!("\n========== 测试 ReturnOne 所有方法 ==========");

    let path = PathBuf::from("examples/ReturnOne.class");
    let class_file = ClassFile::from_file(&path).expect("Failed to load class file");

    println!("类名: {}", class_file.get_class_name().unwrap());
    println!("方法数量: {}", class_file.methods.len());

    // 列出所有方法
    println!("\n=== 方法列表 ===");
    for (i, method) in class_file.methods.iter().enumerate() {
        let name = class_file
            .constant_pool
            .get_utf8(method.name_index)
            .unwrap();
        let descriptor = class_file
            .constant_pool
            .get_utf8(method.descriptor_index)
            .unwrap();
        println!("[{}] {} : {}", i, name, descriptor);
    }

    // 定义要测试的方法和期望结果
    let test_cases = vec![("returnOne", 1), ("addOne", 1), ("calculate", 30)];

    println!("\n=== 执行测试 ===");
    for (method_name, expected) in test_cases {
        println!("\n--- 测试: {} ---", method_name);

        let method = class_file
            .methods
            .iter()
            .find(|m| class_file.constant_pool.get_utf8(m.name_index).unwrap() == method_name)
            .expect(&format!("Method {} not found", method_name));

        let code = method
            .attributes
            .iter()
            .find(|attr| class_file.constant_pool.get_utf8(attr.name_index).unwrap() == "Code")
            .expect("No Code attribute")
            .parse_code_attribute()
            .expect("Failed to parse code");

        let mut interpreter = Interpreter::new();

        match interpreter.execute_method(
            &code.code,
            code.max_locals as usize,
            code.max_stack as usize,
        ) {
            Ok(Some(JvmValue::Int(val))) => {
                println!("  ✓ 返回值: {} (期望: {})", val, expected);
                assert_eq!(val, expected, "{} 返回值不匹配", method_name);
            }
            result => panic!("{} 执行失败: {:?}", method_name, result),
        }
    }

    println!("\n✅ 所有测试通过！");
}

#[test]
fn test_debug_constant_pool() {
    println!("\n========== 调试常量池详情 ==========");

    let path = PathBuf::from("examples/ReturnOne.class");
    let class_file = ClassFile::from_file(&path).expect("Failed to load class file");

    println!("类名: {}", class_file.get_class_name().unwrap());

    // 详细打印常量池
    println!("\n=== 常量池详情 ===");
    println!(
        "总大小: {} (包含索引0)",
        class_file.constant_pool.entries.len()
    );
    println!(
        "有效条目: {} (索引1-{})",
        class_file.constant_pool.entries.len() - 1,
        class_file.constant_pool.entries.len() - 1
    );

    for (i, entry) in class_file.constant_pool.entries.iter().enumerate() {
        if i == 0 {
            println!("\n[0] <保留，不使用>");
            continue;
        }

        match entry {
            Some(e) => {
                println!("\n[{}] {:?}", i, e);

                // 如果是Class，显示其指向的名字
                if let rsjvm::classfile::constant_pool::ConstantPoolEntry::Class { name_index } = e
                {
                    if let Ok(name) = class_file.constant_pool.get_utf8(*name_index) {
                        println!("     └─> 类名: \"{}\"", name);
                    }
                }

                // 如果是MethodRef，显示详情
                if let rsjvm::classfile::constant_pool::ConstantPoolEntry::MethodRef {
                    class_index,
                    name_and_type_index,
                } = e
                {
                    if let Ok(class_name) = class_file.constant_pool.get_class_name(*class_index) {
                        println!("     ├─> 类: \"{}\"", class_name);
                    }
                    if let Ok((method_name, descriptor)) = class_file
                        .constant_pool
                        .get_name_and_type(*name_and_type_index)
                    {
                        println!("     └─> 方法: \"{} : {}\"", method_name, descriptor);
                    }
                }

                // 如果是NameAndType，显示详情
                if let rsjvm::classfile::constant_pool::ConstantPoolEntry::NameAndType {
                    name_index,
                    descriptor_index,
                } = e
                {
                    if let Ok(name) = class_file.constant_pool.get_utf8(*name_index) {
                        println!("     ├─> 名称: \"{}\"", name);
                    }
                    if let Ok(desc) = class_file.constant_pool.get_utf8(*descriptor_index) {
                        println!("     └─> 描述符: \"{}\"", desc);
                    }
                }
            }
            None => {
                println!("\n[{}] <None> (Long/Double占位)", i);
            }
        }
    }

    // 打印方法详情
    println!("\n\n=== 方法详情 ===");
    for (i, method) in class_file.methods.iter().enumerate() {
        let name = class_file
            .constant_pool
            .get_utf8(method.name_index)
            .unwrap();
        let descriptor = class_file
            .constant_pool
            .get_utf8(method.descriptor_index)
            .unwrap();

        println!("\n[{}] {} : {}", i, name, descriptor);
        println!("    访问标志: 0x{:04x}", method.access_flags);
        println!("    属性数量: {}", method.attributes.len());

        for (j, attr) in method.attributes.iter().enumerate() {
            let attr_name = class_file.constant_pool.get_utf8(attr.name_index).unwrap();
            println!(
                "      [{}] 属性: {} (大小: {} bytes)",
                j,
                attr_name,
                attr.info.len()
            );

            if attr_name == "Code" {
                if let Ok(code) = attr.parse_code_attribute() {
                    println!("          max_stack: {}", code.max_stack);
                    println!("          max_locals: {}", code.max_locals);
                    println!(
                        "          字节码 ({} bytes): {:02x?}",
                        code.code.len(),
                        code.code
                    );
                    println!("          异常表: {} 项", code.exception_table.len());
                    println!("          子属性: {} 个", code.attributes.len());
                }
            }
        }
    }
}

#[test]
fn test_debug_return_value() {
    println!("\n========== 调试返回值详情 ==========");

    let path = PathBuf::from("examples/ReturnOne.class");
    let class_file = ClassFile::from_file(&path).expect("Failed to load class file");

    let method = class_file
        .methods
        .iter()
        .find(|m| class_file.constant_pool.get_utf8(m.name_index).unwrap() == "returnOne")
        .expect("Method not found");

    let code = method
        .attributes
        .iter()
        .find(|attr| class_file.constant_pool.get_utf8(attr.name_index).unwrap() == "Code")
        .expect("No Code attribute")
        .parse_code_attribute()
        .expect("Failed to parse code");

    println!("方法: returnOne");
    println!("字节码: {:02x?}", code.code);

    let mut interpreter = Interpreter::new();
    let return_value = interpreter
        .execute_method(
            &code.code,
            code.max_locals as usize,
            code.max_stack as usize,
        )
        .expect("Execution failed");

    println!("\n=== 返回值详情 ===");
    println!("返回值类型: {:?}", return_value);

    match &return_value {
        Some(val) => {
            println!("是否为Some: 是");
            println!("内部值: {:?}", val);

            match val {
                JvmValue::Int(i) => println!("  类型: Int\n  值: {}", i),
                JvmValue::Long(l) => println!("  类型: Long\n  值: {}", l),
                JvmValue::Float(f) => println!("  类型: Float\n  值: {}", f),
                JvmValue::Double(d) => println!("  类型: Double\n  值: {}", d),
                JvmValue::Reference(r) => println!("  类型: Reference\n  值: {:?}", r),
            }
        }
        None => println!("返回值为None (void方法)"),
    }

    // 使用 dbg! 宏显示完整的调试信息
    dbg!(&return_value);
}
