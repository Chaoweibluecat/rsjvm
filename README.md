# RSJVM - Rust 实现的教学型 JVM

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一个用 Rust 从零实现的教学型 Java 虚拟机，用于深入理解 JVM 工作原理。

## 📋 版本信息

**当前版本**: 0.0.1 (Alpha)

### v0.0.1 功能清单

- ✅ 完整的 Class 文件解析器
- ✅ 运行时数据区（方法区/Metaspace、栈、堆）
- ✅ 基础字节码解释器（60+ 指令）
- ✅ 对象模型（new, getfield, putfield）
- ✅ 方法调用（invokestatic, invokespecial, invokevirtual）
- ✅ 控制流指令（if, goto）
- ✅ 简化的类加载器（支持用户类 + 系统类占位）
- ✅ 基础 GC 框架（标记-清除算法）
- ✅ 27 个单元测试和集成测试全部通过

## 🎯 项目目标

这是一个**教学项目**，目标是：

1. **深入理解 JVM 原理** - 通过实现核心功能掌握 JVM 工作机制
2. **学习 Rust 系统编程** - 使用 Rust 处理底层数据结构和内存管理
3. **为技术面试做准备** - 适合 P6 级别（3-5 年经验）Java 工程师面试

**注意**: 这不是生产级 JVM，不支持完整的 Java 标准库。

## ✨ 核心功能

### 1. Class 文件解析器

完整支持 Java 8 Class 文件格式：

```rust
let class_file = ClassFile::from_file("MyClass.class")?;
println!("类名: {}", class_file.get_class_name()?);
println!("方法数: {}", class_file.methods.len());
```

**支持的常量池类型**（14 种）：
- Utf8, Integer, Float, Long, Double
- Class, String, FieldRef, MethodRef, InterfaceMethodRef
- NameAndType, MethodHandle, MethodType, InvokeDynamic

### 2. 运行时数据区

#### Metaspace (方法区)
```rust
pub struct Metaspace {
    classes: HashMap<String, ClassMetadata>,  // 已加载的类
}

pub struct ClassMetadata {
    name: String,
    methods: HashMap<String, MethodMetadata>,  // 方法表
    fields: HashMap<String, FieldMetadata>,    // 字段表
    runtime_pool: RuntimeConstantPool,         // 运行时常量池（带缓存）
}
```

**特性**：
- ✅ 符号引用解析缓存（resolve_method_ref, resolve_field_ref, resolve_class_ref）
- ✅ 快速方法/字段查找
- ✅ 运行时常量池

#### Heap (堆)
```rust
pub struct Heap {
    objects: HashMap<usize, Object>,  // 对象存储
    next_ptr: usize,                  // 下一个分配地址
}

pub struct Object {
    class_name: String,
    fields: HashMap<String, JvmValue>,  // 字段存储
}
```

**特性**：
- ✅ 对象分配 (new)
- ✅ 字段访问 (getfield, putfield)
- ✅ GC 支持（标记-清除）

#### Stack (虚拟机栈)
```rust
pub struct Frame {
    locals: Vec<JvmValue>,        // 局部变量表
    operand_stack: Vec<JvmValue>, // 操作数栈
    class_name: String,           // 动态链接
    code: Vec<u8>,                // 方法字节码
    return_address: Option<usize>,// 返回地址
}
```

### 3. 字节码解释器

支持 **60+ 字节码指令**：

#### 常量指令
`nop`, `iconst_m1`, `iconst_0~5`, `bipush`, `sipush`

#### 加载/存储指令
`iload`, `iload_0~3`, `istore`, `istore_0~3`, `aload`, `aload_0~3`, `astore`, `astore_0~3`

#### 运算指令
`iadd`, `isub`, `imul`, `idiv`

#### 对象操作指令
`new`, `dup`, `getfield`, `putfield`

#### 方法调用指令
- `invokestatic` - 调用静态方法（支持递归）
- `invokespecial` - 调用构造方法、私有方法、super 方法
- `invokevirtual` - 调用实例方法（作弊版支持 println）

#### 控制流指令
`ifeq`, `ifne`, `iflt`, `ifge`, `ifgt`, `ifle`,
`if_icmpeq`, `if_icmpne`, `if_icmplt`, `if_icmpge`, `if_icmpgt`, `if_icmple`, `goto`

#### 返回指令
`ireturn`, `return`

#### 字段访问指令
`getstatic` (作弊版 System.out), `getfield`, `putfield`

### 4. 性能优化

#### 运行时常量池缓存
```rust
pub struct RuntimeConstantPool {
    resolved_methods: HashMap<u16, ResolvedMethodRef>,   // 方法引用缓存
    resolved_fields: HashMap<u16, ResolvedFieldRef>,     // 字段引用缓存
    resolved_classes: HashMap<u16, String>,              // 类引用缓存
}
```

**性能提升**：
- 第一次解析：3 次常量池查找
- 后续访问：1 次 HashMap.get（~5-10x 性能提升）

## 🚀 快速开始

### 安装依赖

```bash
# Rust 1.70+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Java 编译器（用于编译测试代码）
# macOS
brew install openjdk

# Ubuntu
sudo apt install default-jdk
```

### 编译项目

```bash
git clone https://github.com/your-username/rsjvm.git
cd rsjvm
cargo build --release
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行单元测试
cargo test --lib

# 运行集成测试
cargo test --test '*'
```

**测试结果**：
```
running 27 tests
test result: ok. 27 passed; 0 failed
```

### 使用示例

#### 示例 1：运行简单的 Java 程序

```java
// examples/Calculator.java
public class Calculator {
    public static int add(int a, int b) {
        return a + b;
    }

    public static int fibonacci(int n) {
        if (n <= 1) return n;
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}
```

```bash
# 编译
javac examples/Calculator.java

# 运行测试
cargo test test_run_calculate
```

#### 示例 2：对象创建和字段访问

```java
// 支持的对象操作
public class Person {
    private int age;

    public Person(int age) {
        this.age = age;  // putfield
    }

    public int getAge() {
        return this.age;  // getfield
    }
}
```

#### 示例 3：递归调用

```java
public class Test {
    public static int sum(int n) {
        if (n == 0) return 0;
        return n + sum(n - 1);  // 递归调用
    }
}
```

## 📂 项目结构

```
rsjvm/
├── src/
│   ├── classfile/            # Class 文件解析
│   │   ├── mod.rs            # ClassFile 主结构
│   │   └── constant_pool.rs  # 常量池（14 种类型）
│   ├── runtime/              # 运行时数据区
│   │   ├── metaspace.rs      # 方法区 + 类元数据
│   │   ├── frame.rs          # 栈帧（局部变量表 + 操作数栈）
│   │   ├── heap.rs           # 堆（对象分配 + 字段存储）
│   │   └── thread.rs         # 线程（虚拟机栈）
│   ├── interpreter/          # 字节码解释器
│   │   ├── mod.rs            # 解释器主循环（60+ 指令）
│   │   └── instructions.rs   # 指令操作码定义
│   ├── gc/                   # 垃圾回收
│   │   └── mod.rs            # 标记-清除算法
│   └── main.rs               # CLI 入口
├── examples/                 # Java 示例代码
│   ├── Calculator.java
│   ├── ReturnOne.java
│   └── HelloPrintln.java
├── tests/                    # 集成测试
│   ├── interpreter_test.rs   # 解释器测试（8 个）
│   ├── run_test.rs           # 端到端测试（6 个）
│   └── test_invokestatic.rs  # 方法调用测试（3 个）
└── docs/                     # 文档（可选）
```

## 🎓 学习路径

### 阶段 1：Class 文件解析 ✅

**学习重点**：
- Magic Number (0xCAFEBABE) 和版本号
- 常量池的 14 种类型
- 字段表、方法表、属性表

**代码位置**：`src/classfile/`

**练习**：
```bash
javac examples/Simple.java
javap -v examples/Simple.class  # 对比输出
```

### 阶段 2：运行时数据区 ✅

**学习重点**：
- Metaspace（方法区）vs Heap（堆）
- Frame（栈帧）结构
- 局部变量表 vs 操作数栈

**代码位置**：`src/runtime/`

**关键概念**：
```rust
// 栈帧 = 局部变量表 + 操作数栈
let mut frame = Frame::new(max_locals, max_stack);
frame.set_local(0, value);   // 存储到局部变量
frame.push(value);            // 压入操作数栈
```

### 阶段 3：字节码解释器 ✅

**学习重点**：
- 基于栈的虚拟机模型
- 指令格式：opcode + operands
- PC（程序计数器）的作用

**代码位置**：`src/interpreter/mod.rs`

**核心循环**：
```rust
while pc < code.len() {
    let opcode = code[pc];
    match opcode {
        IADD => {
            let v2 = frame.pop_int()?;
            let v1 = frame.pop_int()?;
            frame.push(JvmValue::Int(v1 + v2));
            pc += 1;
        }
        // ...
    }
}
```

### 阶段 4：方法调用机制 ✅

**学习重点**：
- invokestatic（静态方法）
- invokespecial（构造方法、super）
- invokevirtual（实例方法）
- 栈帧切换和返回地址

**代码位置**：`src/interpreter/mod.rs` (INVOKESTATIC, INVOKESPECIAL)

**调用流程**：
```
1. 解析方法引用（符号引用 → 直接引用）
2. 弹出参数
3. 创建新栈帧
4. 设置参数到局部变量表
5. 压入栈帧到线程栈
6. 设置 PC = 0，开始执行
```

### 阶段 5：对象模型 ✅

**学习重点**：
- new 指令（对象分配）
- dup 指令（栈操作）
- getfield/putfield（字段访问）

**代码位置**：`src/runtime/heap.rs`, `src/interpreter/mod.rs`

**对象创建流程**：
```java
Person p = new Person(42);
// 字节码：
new #2          // 分配内存
dup             // 复制引用
bipush 42       // 压入参数
invokespecial #3 // 调用构造方法
astore_1        // 存储到局部变量
```

## 🔬 深入理解

### 符号引用 vs 直接引用

```rust
// 符号引用（在 Class 文件中）
MethodRef {
    class_index: 5,      // 指向常量池 #5
    name_and_type: 12,   // 指向常量池 #12
}

// 直接引用（解析后）
ResolvedMethodRef {
    class_name: "Calculator",
    method_name: "add",
    descriptor: "(II)I",
}
```

### 运行时常量池缓存

```rust
// 第一次调用 Calculator.add(1, 2)
resolve_method_ref(#5)
    → 查找常量池 #5 (MethodRef)
    → 查找常量池 #3 (Class "Calculator")
    → 查找常量池 #8 (Utf8 "Calculator")
    → 查找常量池 #12 (NameAndType)
    → 查找常量池 #13 (Utf8 "add")
    → 查找常量池 #14 (Utf8 "(II)I")
    → 缓存结果到 resolved_methods[#5]

// 第二次调用（循环中）
resolve_method_ref(#5)
    → resolved_methods.get(#5)  // 直接返回！
```

### 动态链接

```rust
pub struct Frame {
    class_name: String,  // ← 这就是动态链接！
    // ...
}

// 使用：
let method_ref = {
    let class_meta = metaspace.get_class(&frame.class_name)?;
    class_meta.resolve_method_ref(index)?
};
```

## 📊 性能数据

### 缓存效果（10,000 次循环）

| 操作 | 无缓存 | 有缓存 | 提升 |
|-----|-------|-------|-----|
| resolve_method_ref | 60,000 次查找 | 6 次查找 + 9,994 次 HashMap.get | ~10x |
| resolve_field_ref | 30,000 次查找 | 3 次查找 + 9,997 次 HashMap.get | ~10x |
| resolve_class_ref | 20,000 次查找 | 2 次查找 + 9,998 次 HashMap.get | ~10x |

## 🎯 适用场景

### ✅ 适合

1. **学习 JVM 原理** - 代码简洁，注释详细
2. **面试准备** - 深度理解类加载、字节码执行、GC
3. **Rust 练习** - 系统编程、所有权、借用检查器
4. **教学演示** - 清晰展示 JVM 核心概念

### ❌ 不适合

1. **运行生产代码** - 不支持完整 Java 标准库
2. **性能测试** - 解释执行，没有 JIT 编译
3. **完整 Java 支持** - 不支持反射、注解、泛型等高级特性

## 🛠️ 开发指南

### 添加新指令

1. 在 `src/interpreter/instructions.rs` 定义操作码
2. 在 `src/interpreter/mod.rs` 的 `execute_instruction_explicit` 添加实现
3. 在 `tests/` 添加测试

示例：
```rust
// 1. 定义操作码
pub const INEG: u8 = 0x74;

// 2. 实现指令
INEG => {
    let value = self.thread.current_frame_mut()?.pop_int()?;
    self.thread.current_frame_mut()?.push(JvmValue::Int(-value));
    self.thread.pc += 1;
}

// 3. 测试
#[test]
fn test_ineg() {
    // ...
}
```

### 调试技巧

```bash
# 查看字节码
javap -c -v MyClass.class

# 运行单个测试并打印日志
RUST_LOG=debug cargo test test_name -- --nocapture

# 调试构建
cargo build && lldb ./target/debug/rsjvm
```

## 📚 参考资源

### 书籍
- 《深入理解 Java 虚拟机》（周志明） - 中文经典
- 《Java 虚拟机规范》（官方） - 权威规范

### 在线资源
- [JVM 规范 SE8](https://docs.oracle.com/javase/specs/jvms/se8/html/)
- [字节码指令集](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html)
- [Class 文件格式](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html)

### 相似项目
- [mini-jvm](https://github.com/guxingke/mini-jvm) - C 语言实现
- [jvm.rs](https://github.com/douchuan/jvm) - Rust 实现

## 🤝 贡献

欢迎 Issue 和 Pull Request！

**贡献方向**：
- 🐛 Bug 修复
- 📝 文档改进
- ✨ 新指令实现
- 🧪 测试用例

## 📝 许可证

MIT License

## 🙏 致谢

感谢所有开源 JVM 项目的启发，以及 Rust 社区的支持。

---

**作者**: [@traviswang](https://github.com/traviswang)
**项目状态**: Alpha - 教学用途
**最后更新**: 2025-12-15
