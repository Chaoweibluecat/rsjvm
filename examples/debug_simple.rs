use rsjvm::runtime::Frame;
use rsjvm::runtime::frame::JvmValue;

fn main() {
    println!("=== 手动执行 iconst_1; ireturn ===\n");

    // 创建栈帧（新架构下 Frame 没有 PC）
    let mut frame = Frame::new(0, 1);  // max_locals=0, max_stack=1
    let mut pc = 0;  // PC 现在在线程级别，这里用局部变量模拟

    println!("初始状态:");
    println!("  PC: {}", pc);
    println!("  栈大小: {}", frame.stack_size());

    // 字节码: 04 ac
    let code = vec![0x04u8, 0xacu8];
    println!("\n字节码: {:02x?}", code);

    // 第1条指令: iconst_1 (0x04)
    println!("\n执行指令 PC={}: iconst_1 (0x{:02x})", pc, code[pc]);
    frame.push(JvmValue::Int(1));
    pc += 1;
    println!("  栈: push(1)");
    println!("  栈大小: {}", frame.stack_size());
    println!("  PC: {}", pc);

    // 第2条指令: ireturn (0xac)
    println!("\n执行指令 PC={}: ireturn (0x{:02x})", pc, code[pc]);
    match frame.pop_int() {
        Ok(val) => {
            println!("  返回值: {}", val);
            println!("  ✓ 成功！");
        }
        Err(e) => {
            println!("  ✗ 错误: {}", e);
        }
    }
}
