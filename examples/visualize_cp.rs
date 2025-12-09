//! 可视化常量池的引用关系
//!
//! 运行: cargo run --example visualize_cp

use rsjvm::classfile::ClassFile;
use rsjvm::classfile::constant_pool::ConstantPoolEntry;

fn main() -> anyhow::Result<()> {
    let class_file = ClassFile::from_file("examples/ReturnOne.class")?;

    println!("=== 常量池引用关系可视化 ===\n");

    // 找到第一个方法的第一条 invokespecial 指令
    let init_method = &class_file.methods[0];
    let init_name = class_file.constant_pool.get_utf8(init_method.name_index)?;
    println!("分析方法: {}", init_name);

    let code_attr = init_method.attributes[0].parse_code_attribute()?;
    println!("字节码: {:02x?}", code_attr.code);
    println!("解码: aload_0, invokespecial #1, return\n");

    println!("=== 追踪常量池 #1 的引用链 ===\n");

    // #1 是 MethodRef
    match class_file.constant_pool.get(1)? {
        ConstantPoolEntry::MethodRef { class_index, name_and_type_index } => {
            println!("[#1] MethodRef");
            println!("  ├─ class_index: #{}", class_index);
            println!("  └─ name_and_type_index: #{}", name_and_type_index);

            // 追踪 class_index
            println!("\n追踪类引用 #{}:", class_index);
            match class_file.constant_pool.get(*class_index)? {
                ConstantPoolEntry::Class { name_index } => {
                    println!("  [#{}] Class", class_index);
                    println!("    └─ name_index: #{}", name_index);

                    // 追踪类名
                    println!("\n  追踪类名 #{}:", name_index);
                    let class_name = class_file.constant_pool.get_utf8(*name_index)?;
                    println!("    [#{}] Utf8(\"{}\")", name_index, class_name);
                }
                _ => {}
            }

            // 追踪 name_and_type_index
            println!("\n追踪名称和类型 #{}:", name_and_type_index);
            match class_file.constant_pool.get(*name_and_type_index)? {
                ConstantPoolEntry::NameAndType { name_index, descriptor_index } => {
                    println!("  [#{}] NameAndType", name_and_type_index);
                    println!("    ├─ name_index: #{}", name_index);
                    println!("    └─ descriptor_index: #{}", descriptor_index);

                    // 追踪方法名
                    println!("\n  追踪方法名 #{}:", name_index);
                    let method_name = class_file.constant_pool.get_utf8(*name_index)?;
                    println!("    [#{}] Utf8(\"{}\")", name_index, method_name);

                    // 追踪描述符
                    println!("\n  追踪描述符 #{}:", descriptor_index);
                    let descriptor = class_file.constant_pool.get_utf8(*descriptor_index)?;
                    println!("    [#{}] Utf8(\"{}\")", descriptor_index, descriptor);
                }
                _ => {}
            }

            println!("\n=== 完整引用链 ===");
            let class_name = class_file.constant_pool.get_class_name(*class_index)?;
            let (method_name, descriptor) = class_file.constant_pool.get_name_and_type(*name_and_type_index)?;
            println!("invokespecial #1 调用的方法是:");
            println!("  {}.{}{}", class_name, method_name, descriptor);
            println!("\n解释: 调用父类 java.lang.Object 的构造函数");
        }
        _ => println!("不是MethodRef"),
    }

    // 统计常量池的类型分布
    println!("\n\n=== 常量池类型统计 ===");
    let mut type_counts = std::collections::HashMap::new();
    for entry in &class_file.constant_pool.entries {
        if let Some(e) = entry {
            let type_name = match e {
                ConstantPoolEntry::Utf8(_) => "Utf8",
                ConstantPoolEntry::Integer(_) => "Integer",
                ConstantPoolEntry::Float(_) => "Float",
                ConstantPoolEntry::Long(_) => "Long",
                ConstantPoolEntry::Double(_) => "Double",
                ConstantPoolEntry::Class { .. } => "Class",
                ConstantPoolEntry::String { .. } => "String",
                ConstantPoolEntry::FieldRef { .. } => "FieldRef",
                ConstantPoolEntry::MethodRef { .. } => "MethodRef",
                ConstantPoolEntry::InterfaceMethodRef { .. } => "InterfaceMethodRef",
                ConstantPoolEntry::NameAndType { .. } => "NameAndType",
                ConstantPoolEntry::MethodHandle { .. } => "MethodHandle",
                ConstantPoolEntry::MethodType { .. } => "MethodType",
                ConstantPoolEntry::InvokeDynamic { .. } => "InvokeDynamic",
            };
            *type_counts.entry(type_name).or_insert(0) += 1;
        }
    }

    for (type_name, count) in type_counts.iter() {
        println!("{:20} : {} 个", type_name, count);
    }

    Ok(())
}
