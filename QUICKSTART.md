# RSJVM 快速入门指南

## 项目已完成 ✅

恭喜！RSJVM的基础框架已经搭建完成，并且核心功能已经可以工作了。

### 已实现的功能

#### 1. Class文件解析器（完整）
- ✅ 解析magic number、版本号
- ✅ 解析常量池（14种常量类型）
- ✅ 解析字段、方法、属性
- ✅ 解析Code属性

#### 2. 字节码解释器（基础）
已实现的指令：
- ✅ **常量指令**: `iconst_m1`, `iconst_0~5`, `bipush`, `sipush`
- ✅ **加载指令**: `iload_0~3`
- ✅ **存储指令**: `istore_0~3`
- ✅ **运算指令**: `iadd`, `isub`, `imul`, `idiv`
- ✅ **返回指令**: `ireturn`, `return`

#### 3. 运行时数据区
- ✅ 栈帧（Frame）with 局部变量表 + 操作数栈
- ✅ 堆（Heap）with 对象分配和管理
- ✅ 线程（JvmThread）with 虚拟机栈

#### 4. 测试完整性
- ✅ 8个单元测试全部通过
- ✅ 包括边界情况（除零错误）

---

## 现在就开始玩！

### 步骤1：编译项目

```bash
cargo build --release
```

### 步骤2：解析一个class文件

```bash
# 编译示例Java文件
javac examples/ReturnOne.java

# 解析class文件（基本信息）
./target/release/rsjvm parse examples/ReturnOne.class

# 查看详细信息（包括字节码）
./target/release/rsjvm parse examples/ReturnOne.class --verbose
```

输出示例：
```
=== 基本信息 ===
魔数: 0xCAFEBABE
版本: 61.0 (Java (version 61))
类名: ReturnOne
父类: java/lang/Object

=== 方法 (4) ===
  [1] returnOne : ()I
      max_stack: 1
      max_locals: 0
      code_length: 2
      bytecode:
        0000  04 ac    # iconst_1, ireturn
```

### 步骤3：运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_simple_add
```

### 步骤4：理解字节码

查看 `examples/BYTECODE_ANALYSIS.md`，这份文档详细解释了：
- `PrintOne.java` - `System.out.println(1)` 的字节码
- `ReturnOne.java` - 三个可运行的简单方法
- 字节码指令速查表
- 与C编译器的对比

### 步骤5：实验！

#### 实验1：手动执行字节码

创建你自己的测试：

```rust
// tests/my_test.rs
use rsjvm::interpreter::Interpreter;

#[test]
fn test_my_bytecode() {
    let bytecode = vec![
        0x04, // iconst_1
        0x05, // iconst_2
        0x60, // iadd
        0xac, // ireturn
    ];

    let mut interpreter = Interpreter::new();
    assert!(interpreter.execute_method(&bytecode, 0, 2).is_ok());
}
```

#### 实验2：写一个Java方法并分析

```java
// examples/MyTest.java
public class MyTest {
    public static int fibonacci(int n) {
        if (n <= 1) return n;
        return fibonacci(n-1) + fibonacci(n-2);
    }
}
```

编译并解析：
```bash
javac examples/MyTest.java
./target/release/rsjvm parse examples/MyTest.class -v
```

观察递归调用的字节码！

#### 实验3：对比不同写法的字节码

```java
// 写法1
public static int sum1() {
    return 1 + 2;
}

// 写法2
public static int sum2() {
    int a = 1;
    int b = 2;
    return a + b;
}
```

看看编译器是否会优化？

---

## 学习路径

### 第1周：理解Class文件和字节码

**目标**：能够看懂任何简单Java方法的字节码

**任务**：
1. 阅读 `examples/BYTECODE_ANALYSIS.md`
2. 写10个简单方法，查看它们的字节码
3. 手动在纸上执行这些字节码（模拟栈和局部变量表）

**检验**：能够在不运行的情况下，预测字节码的执行结果

### 第2周：扩展解释器

**目标**：支持更多指令

**任务**：
1. 实现long、float、double类型的指令
2. 实现数组操作指令（`newarray`, `iaload`, `iastore`）
3. 实现条件跳转指令（`ifeq`, `ifne`, `if_icmpgt`等）
4. 实现循环（配合goto指令）

**检验**：能够运行包含循环和条件的方法

### 第3周：对象和方法调用

**目标**：支持OOP

**任务**：
1. 实现`new`指令 - 创建对象
2. 实现`getfield`/`putfield` - 访问字段
3. 实现`invokevirtual` - 调用实例方法
4. 实现`invokestatic` - 调用静态方法

**检验**：能够运行包含对象创建和方法调用的代码

### 第4周：GC和优化

**目标**：完善垃圾回收

**任务**：
1. 完善标记-清除算法
2. 实现引用计数作为对比
3. 添加性能统计
4. 尝试简单的JIT优化

**检验**：能够正确回收不再使用的对象

---

## 当前可以运行的例子

以下方法的字节码**现在就可以用解释器运行**：

### ✅ 例1：直接返回常量
```java
public static int returnOne() {
    return 1;
}
```
字节码: `04 ac` (iconst_1, ireturn)

### ✅ 例2：简单加法
```java
public static int add() {
    int a = 1;
    int b = 2;
    return a + b;
}
```

### ✅ 例3：四则运算
```java
public static int calculate() {
    int a = 10;
    int b = 20;
    int c = a + b;   // 加法
    int d = c - 5;   // 减法
    int e = d * 2;   // 乘法
    int f = e / 3;   // 除法
    return f;
}
```

### ❌ 暂时不能运行（需要更多指令）

```java
// 需要循环（goto）
public static int sum(int n) {
    int result = 0;
    for (int i = 0; i <= n; i++) {
        result += i;
    }
    return result;
}

// 需要对象创建（new）
public static Object createObject() {
    return new Object();
}

// 需要方法调用（invokevirtual）
public static void println() {
    System.out.println("Hello");
}
```

---

## 调试技巧

### 技巧1：添加日志

在解释器中添加日志输出：

```rust
// src/interpreter/mod.rs
fn execute_instruction(...) -> Result<bool> {
    log::debug!("PC={}, opcode=0x{:02x}", frame.pc, opcode);
    match opcode {
        ...
    }
}
```

运行时启用日志：
```bash
RUST_LOG=debug cargo test test_simple_add -- --nocapture
```

### 技巧2：查看栈状态

在Frame中添加调试方法：

```rust
impl Frame {
    pub fn dump(&self) {
        println!("PC: {}", self.pc);
        println!("Stack: {:?}", self.operand_stack);
        println!("Locals: {:?}", self.local_vars);
    }
}
```

### 技巧3：对比javap

使用官方工具查看字节码：

```bash
javap -c -v examples/ReturnOne.class
```

对比我们的解析结果，验证正确性。

---

## 下一步做什么？

### 选项A：继续深入JVM
- 实现更多字节码指令
- 添加异常处理
- 实现接口和继承
- 研究JIT编译

### 选项B：横向扩展
- 实现一个简单的Java编译器（源码→字节码）
- 设计自己的字节码虚拟机
- 研究其他语言的VM（如Python的CPython、Lua的VM）

### 选项C：实际应用
- 写一个字节码增强工具（类似CGLIB）
- 写一个性能分析工具
- 写一个Java代码混淆器

---

## 常见问题

### Q: 为什么不能运行System.out.println？
A: 需要实现native方法支持和标准库。这很复杂，但可以作为长期目标。

### Q: 我的JVM能通过TCK吗？
A: 不能。这是学习项目，不是生产JVM。通过TCK需要数千小时的工作。

### Q: 性能如何？
A: 很慢，因为是纯解释执行。OpenJDK有JIT编译器，我们没有。

### Q: 可以用来运行实际的Java程序吗？
A: 不行。缺少标准库、JIT、完整的指令集等。但可以运行简单的算法。

---

## 资源

### 必读文档
- [JVM规范（Java 8）](https://docs.oracle.com/javase/specs/jvms/se8/html/)
- [字节码指令集](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html)

### 推荐书籍
- 《深入理解Java虚拟机》（周志明）
- 《Java虚拟机规范》（官方）

### 参考项目
- [mini-jvm](https://github.com/guxingke/mini-jvm)
- [jvm.rs](https://github.com/douchuan/jvm)
- [OpenJDK](https://github.com/openjdk/jdk)

---

## 祝你学习愉快！

现在开始你的JVM探索之旅吧！记住：

> **The best way to understand a system is to build one.**

有问题随时查看代码注释，每个模块都有详细的学习要点说明。

加油！🚀
