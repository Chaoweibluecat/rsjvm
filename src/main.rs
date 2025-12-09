//! # RSJVM - Rust实现的学习型JVM
//!
//! 命令行工具，用于加载和执行Java class文件

use anyhow::Result;
use clap::Parser;
use rsjvm::classfile::ClassFile;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rsjvm")]
#[command(about = "Rust实现的学习型JVM", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    /// 解析并显示class文件信息
    Parse {
        /// class文件路径
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// 显示详细信息
        #[arg(short, long)]
        verbose: bool,
    },

    /// 运行class文件中的方法
    Run {
        /// class文件路径
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// 要运行的方法名（如果不指定，则自动查找main方法）
        #[arg(short, long)]
        method: Option<String>,

        /// 命令行参数（传递给main方法，暂未实现）
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// 显示版本信息
    Version,
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { file, verbose } => {
            parse_class_file(&file, verbose)?;
        }
        Commands::Run { file, method, args } => {
            run_class_file(&file, method.as_deref(), args)?;
        }
        Commands::Version => {
            println!("RSJVM version {}", env!("CARGO_PKG_VERSION"));
            println!("一个用于学习JVM原理的Rust实现");
        }
    }

    Ok(())
}

/// 解析并显示class文件信息
fn parse_class_file(path: &PathBuf, verbose: bool) -> Result<()> {
    println!("正在解析: {:?}\n", path);

    let class_file = ClassFile::from_file(path)?;

    // 基本信息
    println!("=== 基本信息 ===");
    println!("魔数: 0x{:08X}", class_file.magic);
    println!(
        "版本: {}.{} ({})",
        class_file.major_version,
        class_file.minor_version,
        class_file.get_java_version()
    );
    println!("类名: {}", class_file.get_class_name()?);
    println!("父类: {}", class_file.get_super_class_name()?);
    println!("访问标志: 0x{:04X}", class_file.access_flags);

    // 接口
    if !class_file.interfaces.is_empty() {
        println!("\n=== 接口 ({}) ===", class_file.interfaces.len());
        for (i, &interface_index) in class_file.interfaces.iter().enumerate() {
            let interface_name = class_file.constant_pool.get_class_name(interface_index)?;
            println!("  [{}] {}", i, interface_name);
        }
    }

    // 字段
    println!("\n=== 字段 ({}) ===", class_file.fields.len());
    for (i, field) in class_file.fields.iter().enumerate() {
        let name = class_file.constant_pool.get_utf8(field.name_index)?;
        let descriptor = class_file.constant_pool.get_utf8(field.descriptor_index)?;
        println!("  [{}] {} : {}", i, name, descriptor);
    }

    // 方法
    println!("\n=== 方法 ({}) ===", class_file.methods.len());
    for (i, method) in class_file.methods.iter().enumerate() {
        let name = class_file.constant_pool.get_utf8(method.name_index)?;
        let descriptor = class_file.constant_pool.get_utf8(method.descriptor_index)?;
        println!("  [{}] {} : {}", i, name, descriptor);

        if verbose {
            // 尝试解析Code属性
            for attr in &method.attributes {
                let attr_name = class_file.constant_pool.get_utf8(attr.name_index)?;
                if attr_name == "Code" {
                    if let Ok(code_attr) = attr.parse_code_attribute() {
                        println!("      max_stack: {}", code_attr.max_stack);
                        println!("      max_locals: {}", code_attr.max_locals);
                        println!("      code_length: {}", code_attr.code.len());

                        if verbose {
                            println!("      bytecode:");
                            print_bytecode(&code_attr.code);
                        }
                    }
                }
            }
        }
    }

    // 常量池（详细模式）
    if verbose {
        println!(
            "\n=== 常量池 ({}) ===",
            class_file.constant_pool.entries.len() - 1
        );
        for (i, entry) in class_file.constant_pool.entries.iter().enumerate() {
            if i == 0 {
                continue; // 跳过索引0
            }
            if let Some(entry) = entry {
                println!("  [{}] {:?}", i, entry);
            }
        }
    }

    Ok(())
}

/// 打印字节码（十六进制）
fn print_bytecode(code: &[u8]) {
    for (i, chunk) in code.chunks(16).enumerate() {
        print!("        {:04x}  ", i * 16);
        for byte in chunk {
            print!("{:02x} ", byte);
        }
        println!();
    }
}

/// 查找main方法
fn find_main_method(class_file: &ClassFile) -> Result<&rsjvm::classfile::MethodInfo> {
    const ACC_PUBLIC: u16 = 0x0001;
    const ACC_STATIC: u16 = 0x0008;

    for method in &class_file.methods {
        let name = class_file.constant_pool.get_utf8(method.name_index)?;
        let descriptor = class_file.constant_pool.get_utf8(method.descriptor_index)?;

        // 检查是否是main方法
        if name == "main" && descriptor == "([Ljava/lang/String;)V" {
            // 检查访问标志：必须是 public static
            if (method.access_flags & ACC_PUBLIC) != 0 && (method.access_flags & ACC_STATIC) != 0 {
                return Ok(method);
            }
        }
    }

    Err(anyhow::anyhow!(
        "找不到 public static void main(String[] args) 方法"
    ))
}

/// 运行class文件中的方法
fn run_class_file(path: &PathBuf, method_name: Option<&str>, args: Vec<String>) -> Result<()> {
    use rsjvm::interpreter::Interpreter;
    use rsjvm::runtime::frame::JvmValue;

    println!("正在加载: {:?}\n", path);

    let class_file = ClassFile::from_file(path)?;
    let class_name = class_file.get_class_name()?;

    println!("类名: {}", class_name);

    // 查找方法
    let (method, method_to_run) = if let Some(name) = method_name {
        // 用户指定了方法名
        println!("查找方法: {}", name);
        let mut found_method = None;
        for method in &class_file.methods {
            let method_name = class_file.constant_pool.get_utf8(method.name_index)?;
            if method_name == name {
                found_method = Some(method);
                break;
            }
        }
        let method = found_method.ok_or_else(|| anyhow::anyhow!("方法未找到: {}", name))?;
        (method, name.to_string())
    } else {
        // 自动查找main方法
        println!("自动查找main方法...");
        let method = find_main_method(&class_file)?;
        println!("✓ 找到main方法");
        (method, "main".to_string())
    };

    if !args.is_empty() {
        println!("命令行参数: {:?} (注意：当前版本暂不支持传递参数)", args);
    }

    let descriptor = class_file.constant_pool.get_utf8(method.descriptor_index)?;
    println!("方法签名: {} : {}", method_to_run, descriptor);

    // 查找Code属性
    let mut code_attr = None;
    for attr in &method.attributes {
        let attr_name = class_file.constant_pool.get_utf8(attr.name_index)?;
        if attr_name == "Code" {
            code_attr = Some(attr.parse_code_attribute()?);
            break;
        }
    }

    let code = code_attr.ok_or_else(|| anyhow::anyhow!("方法没有Code属性"))?;

    println!("\n=== 方法信息 ===");
    println!("max_stack: {}", code.max_stack);
    println!("max_locals: {}", code.max_locals);
    println!("code_length: {}", code.code.len());
    println!("\n字节码:");
    print_bytecode(&code.code);

    // 执行方法
    println!("\n=== 开始执行 ===");
    let mut interpreter = Interpreter::new();

    match interpreter.execute_method(
        &code.code,
        code.max_locals as usize,
        code.max_stack as usize,
    ) {
        Ok(return_value) => {
            println!("✓ 执行成功！");

            // 显示返回值
            if let Some(val) = return_value {
                println!("\n=== 返回值 ===");
                match val {
                    JvmValue::Int(i) => println!("int: {}", i),
                    JvmValue::Long(l) => println!("long: {}", l),
                    JvmValue::Float(f) => println!("float: {}", f),
                    JvmValue::Double(d) => println!("double: {}", d),
                    JvmValue::Reference(r) => println!("reference: {:?}", r),
                }
            } else {
                println!("\n方法无返回值 (void)");
            }
        }
        Err(e) => {
            println!("✗ 执行失败: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
