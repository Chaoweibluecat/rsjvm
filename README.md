# RSJVM - Rust实现的学习型JVM

一个用于学习Java虚拟机原理的Rust实现项目。

## 项目目标

这是一个**学习性质**的项目，旨在通过实现JVM的核心功能来深入理解：
- Java字节码结构
- JVM运行时数据区
- 字节码解释执行
- 垃圾回收原理
- 类加载机制

**注意**：这不是一个生产级别的JVM实现，而是一个教学工具。

## 功能特性

### 已实现
- ✅ Class文件解析器
  - 解析class文件结构
  - 读取常量池
  - 解析字段、方法、属性
- ✅ 运行时数据区
  - 栈帧（Frame）
  - 堆（Heap）
  - 线程（Thread）
- ✅ 基础字节码解释器
  - 常量指令（iconst, bipush, sipush等）
  - 加载/存储指令（iload, istore等）
  - 运算指令（iadd, isub, imul, idiv）
  - 返回指令（ireturn, return）
- ✅ 类加载器框架
- ✅ 简单GC框架（标记-清除算法）

### 计划实现
- ⏳ 更多字节码指令
- ⏳ 对象创建和方法调用
- ⏳ 异常处理
- ⏳ 本地方法支持

## 项目结构

```
rsjvm/
├── src/
│   ├── main.rs              # 命令行工具入口
│   ├── lib.rs               # 库入口
│   ├── classfile/           # Class文件解析
│   │   ├── mod.rs           # ClassFile结构定义
│   │   ├── parser.rs        # 字节码解析器
│   │   ├── constant_pool.rs # 常量池
│   │   └── attribute.rs     # 属性
│   ├── runtime/             # 运行时数据区
│   │   ├── mod.rs
│   │   ├── frame.rs         # 栈帧
│   │   ├── heap.rs          # 堆
│   │   └── thread.rs        # 线程
│   ├── interpreter/         # 字节码解释器
│   │   ├── mod.rs           # 解释器核心
│   │   └── instructions.rs  # 指令定义
│   ├── classloader/         # 类加载器
│   │   └── mod.rs
│   └── gc/                  # 垃圾回收
│       └── mod.rs
├── examples/                # 示例Java文件
└── tests/                   # 测试
```

## 快速开始

### 编译项目

```bash
cargo build --release
```

### 使用示例

#### 1. 解析class文件

创建一个简单的Java类：

```java
// examples/Simple.java
public class Simple {
    public static int add(int a, int b) {
        return a + b;
    }
}
```

编译并解析：

```bash
# 编译Java文件
javac examples/Simple.java

# 解析class文件
cargo run -- parse examples/Simple.class

# 显示详细信息（包括字节码）
cargo run -- parse examples/Simple.class --verbose
```

#### 2. 查看版本信息

```bash
cargo run -- version
```

## 学习路径

### 第一阶段：理解Class文件格式 ✅

**学习目标**：
- 理解Java字节码文件结构
- 掌握常量池的作用
- 了解字段和方法的表示

**实践练习**：
1. 编写简单的Java类并查看其字节码
2. 使用`parse`命令分析不同复杂度的class文件
3. 观察不同Java特性在字节码层面的表示

**相关代码**：
- `src/classfile/`目录下的所有文件

### 第二阶段：实现字节码解释器 ⏳

**学习目标**：
- 理解JVM的栈式虚拟机模型
- 掌握基本字节码指令的执行
- 了解局部变量表和操作数栈

**实践练习**：
1. 手动构造简单的字节码序列
2. 跟踪指令执行过程
3. 实现更多字节码指令

**相关代码**：
- `src/interpreter/mod.rs`
- `src/runtime/frame.rs`

### 第三阶段：对象和方法调用 ⏳

**学习目标**：
- 理解对象的创建和内存分配
- 掌握方法调用机制
- 了解动态分派

**相关代码**：
- `src/runtime/heap.rs`
- `src/interpreter/mod.rs`（方法调用相关）

### 第四阶段：垃圾回收 ⏳

**学习目标**：
- 理解GC算法原理
- 掌握可达性分析
- 了解GC Roots概念

**相关代码**：
- `src/gc/mod.rs`

## 核心概念

### 1. Class文件结构

Class文件是JVM的输入格式，包含：
- 魔数：`0xCAFEBABE`
- 版本号：主版本号52表示Java 8
- 常量池：存储字面量和符号引用
- 访问标志：public、final等修饰符
- 类索引、父类索引、接口索引
- 字段表、方法表、属性表

### 2. 运行时数据区

JVM运行时数据区包括：
- **程序计数器**：当前线程执行的字节码位置
- **虚拟机栈**：线程私有，存储栈帧
- **堆**：所有对象实例的存储区域
- **方法区**：类的元数据存储

### 3. 栈帧结构

每个方法调用都创建一个栈帧，包含：
- **局部变量表**：存储方法参数和局部变量
- **操作数栈**：字节码指令的工作区
- **动态链接**：指向运行时常量池的引用
- **返回地址**：方法返回后的执行位置

### 4. 字节码指令

JVM指令特点：
- 单字节操作码（opcode）
- 基于栈的虚拟机
- 类型特定的指令（如iadd、ladd）

## 开发指南

### 运行测试

```bash
cargo test
```

### 查看文档

```bash
cargo doc --open
```

### 代码规范

项目使用标准的Rust代码风格：

```bash
cargo fmt
cargo clippy
```

## 学习资源

### 推荐书籍
- 《深入理解Java虚拟机》（周志明）
- 《Java虚拟机规范》（官方文档）

### 在线资源
- [JVM规范](https://docs.oracle.com/javase/specs/jvms/se8/html/)
- [字节码指令集](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html)

## 贡献

这是一个学习项目，欢迎：
- 提出问题和建议
- 改进文档和注释
- 实现更多功能

## 许可证

MIT License

## 致谢

本项目受以下开源项目启发：
- [mini-jvm](https://github.com/guxingke/mini-jvm)
- [jvm.rs](https://github.com/douchuan/jvm)
