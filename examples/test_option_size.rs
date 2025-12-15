use std::mem;

fn main() {
    println!("=== usize 和 Option<usize> 的大小 ===\n");

    // 基础类型
    println!("基础类型:");
    println!("  usize:           {} bytes", mem::size_of::<usize>());
    println!("  u8:              {} bytes", mem::size_of::<u8>());
    println!("  u64:             {} bytes", mem::size_of::<u64>());

    println!("\nOption类型:");
    println!("  Option<usize>:   {} bytes", mem::size_of::<Option<usize>>());
    println!("  Option<u8>:      {} bytes", mem::size_of::<Option<u8>>());
    println!("  Option<u64>:     {} bytes", mem::size_of::<Option<u64>>());
    println!("  Option<bool>:    {} bytes", mem::size_of::<Option<bool>>());

    println!("\n指针类型的Option:");
    println!("  Option<&i32>:    {} bytes", mem::size_of::<Option<&i32>>());
    println!("  Option<Box<i32>>:{} bytes", mem::size_of::<Option<Box<i32>>>());
    println!("  &i32:            {} bytes", mem::size_of::<&i32>());
    println!("  Box<i32>:        {} bytes", mem::size_of::<Box<i32>>());

    println!("\n系统信息:");
    println!("  指针大小:        {} bytes", mem::size_of::<*const ()>());
    println!("  是64位系统:      {}", mem::size_of::<usize>() == 8);

    println!("\n=== 为什么 Option<usize> 是 16 bytes? ===\n");

    // 实际测试：创建Option值
    let none_val: Option<usize> = None;
    let some_val: Option<usize> = Some(42);

    println!("None:  {:?} - 占用 {} bytes", none_val, mem::size_of_val(&none_val));
    println!("Some:  {:?} - 占用 {} bytes", some_val, mem::size_of_val(&some_val));

    println!("\n原因分析:");
    println!("Option<usize> 的内存布局:");
    println!("  ┌─────────────────┐");
    println!("  │ tag: 8 bytes    │  ← 判别标签（None或Some）");
    println!("  ├─────────────────┤");
    println!("  │ data: 8 bytes   │  ← usize数据（如果是Some）");
    println!("  └─────────────────┘");
    println!("  总计: 16 bytes");

    println!("\n对比其他Option:");
    println!("Option<bool> = {} bytes (bool只需1 byte，但tag需要额外空间)",
             mem::size_of::<Option<bool>>());
    println!("Option<u8> = {} bytes (u8只需1 byte，但tag需要额外空间)",
             mem::size_of::<Option<u8>>());

    println!("\n但是！指针类型的Option有优化:");
    println!("Option<&i32>只需 {} bytes (利用null pointer optimization)",
             mem::size_of::<Option<&i32>>());
    println!("  因为引用永远不是NULL，所以可以用0表示None！");

    // 查看实际内存内容
    println!("\n=== 实际内存内容 ===");
    let none: Option<usize> = None;
    let some: Option<usize> = Some(0x1234567890ABCDEF);

    unsafe {
        let none_bytes = std::slice::from_raw_parts(
            &none as *const _ as *const u8,
            mem::size_of::<Option<usize>>()
        );
        let some_bytes = std::slice::from_raw_parts(
            &some as *const _ as *const u8,
            mem::size_of::<Option<usize>>()
        );

        println!("None 的内存内容:");
        print!("  ");
        for (i, &byte) in none_bytes.iter().enumerate() {
            print!("{:02X} ", byte);
            if i == 7 { print!(" | "); }
        }
        println!("\n       ^^^^^^^^^^^^^^^   ^^^^^^^^^^^^^^^^");
        println!("           tag部分           data部分");

        println!("\nSome(0x1234567890ABCDEF) 的内存内容:");
        print!("  ");
        for (i, &byte) in some_bytes.iter().enumerate() {
            print!("{:02X} ", byte);
            if i == 7 { print!(" | "); }
        }
        println!("\n       ^^^^^^^^^^^^^^^   ^^^^^^^^^^^^^^^^");
        println!("           tag部分           data部分");
    }
}
