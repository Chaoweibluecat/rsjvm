use std::mem;

#[derive(Debug, Clone)]
pub enum JvmValue {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Reference(Option<usize>),
}

fn main() {
    println!("=== JvmValue 内存布局分析 ===\n");

    // 各个variant的数据大小
    println!("基础类型大小:");
    println!("  i32:          {} bytes", mem::size_of::<i32>());
    println!("  i64:          {} bytes", mem::size_of::<i64>());
    println!("  f32:          {} bytes", mem::size_of::<f32>());
    println!("  f64:          {} bytes", mem::size_of::<f64>());
    println!("  Option<usize>:{} bytes", mem::size_of::<Option<usize>>());

    println!("\nJvmValue枚举大小:");
    println!("  整个枚举:     {} bytes", mem::size_of::<JvmValue>());
    println!("  对齐要求:     {} bytes", mem::align_of::<JvmValue>());

    println!("\n实际的内存布局:");
    println!("  判别标签(discriminant): 通常 1-8 bytes");
    println!("  数据部分: max(各variant) = {} bytes",
             mem::size_of::<i64>().max(mem::size_of::<Option<usize>>()));
    println!("  加上padding对齐");

    println!("\n创建不同variant:");
    let v_int = JvmValue::Int(42);
    let v_long = JvmValue::Long(42);
    let v_float = JvmValue::Float(3.14);
    let v_double = JvmValue::Double(3.14);
    let v_ref = JvmValue::Reference(Some(0));

    println!("  Int:       {:?} - 占用 {} bytes", v_int, mem::size_of_val(&v_int));
    println!("  Long:      {:?} - 占用 {} bytes", v_long, mem::size_of_val(&v_long));
    println!("  Float:     {:?} - 占用 {} bytes", v_float, mem::size_of_val(&v_float));
    println!("  Double:    {:?} - 占用 {} bytes", v_double, mem::size_of_val(&v_double));
    println!("  Reference: {:?} - 占用 {} bytes", v_ref, mem::size_of_val(&v_ref));

    println!("\nVec<JvmValue> 分析:");
    let values = vec![
        JvmValue::Int(1),
        JvmValue::Long(2),
        JvmValue::Float(3.0),
    ];
    println!("  Vec元素数量:   {}", values.len());
    println!("  Vec容量:       {}", values.capacity());
    println!("  总内存占用:    {} bytes", values.capacity() * mem::size_of::<JvmValue>());
    println!("  Vec本身结构:   {} bytes (ptr + len + cap)", mem::size_of::<Vec<JvmValue>>());

    println!("\n内存浪费分析:");
    let waste_int = mem::size_of::<JvmValue>() - mem::size_of::<i32>() - 1; // -1是discriminant
    println!("  存储Int浪费:   ~{} bytes", waste_int);
    println!("  浪费率:        {:.1}%",
             (waste_int as f64 / mem::size_of::<JvmValue>() as f64) * 100.0);
}
