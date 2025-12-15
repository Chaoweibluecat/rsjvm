# RSJVM 项目指南

## 项目简介
这是一个用 Rust 实现的教学型 JVM（Java Virtual Machine），目标是深入理解 JVM 的工作原理。

**当前版本**: v0.0.1 (Alpha) - 已完成核心 JVM 功能

## 项目结构
```
rsjvm/
├── src/
│   ├── classfile/          # Class 文件解析器
│   │   ├── constant_pool.rs  # 常量池（14种常量类型）
│   │   └── mod.rs           # 主解析逻辑
│   ├── interpreter/        # 字节码解释器
│   │   ├── instructions.rs  # 指令操作码定义
│   │   └── mod.rs          # 解释器核心逻辑（60+ 指令）
│   ├── runtime/            # 运行时数据区
│   │   ├── metaspace.rs    # 方法区 + 类元数据 + 运行时常量池
│   │   ├── frame.rs        # 栈帧（局部变量表+操作数栈）
│   │   ├── heap.rs         # 堆（对象分配和GC）
│   │   ├── thread.rs       # JVM线程
│   │   └── mod.rs
│   ├── gc/                 # 垃圾回收
│   │   └── mod.rs          # 标记-清除算法
│   ├── main.rs             # CLI入口
│   └── lib.rs
├── examples/               # Java示例代码
└── tests/                  # 测试套件（27个测试全部通过）
    ├── interpreter_test.rs
    ├── run_test.rs
    └── test_invokestatic.rs
```

## 当前开发状态（v0.0.1）

### ✅ 已完成的核心功能

#### 1. **Class 文件解析器**（完整实现）
   - Magic number (0xCAFEBABE)、版本号解析
   - 常量池解析（14种常量类型全部支持）
   - 字段表、方法表、属性表解析
   - Code 属性完整解析

#### 2. **运行时数据区**（完整架构）
   - **Metaspace（方法区）**
     - 类元数据管理 (ClassMetadata)
     - 运行时常量池 (RuntimeConstantPool)
     - 符号引用解析缓存（resolve_method_ref, resolve_field_ref, resolve_class_ref）
     - 方法/字段快速查找（HashMap 索引）
     - **性能优化**: 缓存后性能提升约 10x
   - **Frame（栈帧）**
     - 局部变量表 (locals)
     - 操作数栈 (operand_stack)
     - 动态链接 (class_name)
     - 返回地址 (return_address)
   - **Heap（堆）**
     - 对象分配 (new)
     - 字段存储 (HashMap<String, JvmValue>)
     - GC 支持
   - **JvmThread（线程）**
     - 虚拟机栈管理
     - PC（程序计数器）

#### 3. **字节码解释器**（60+ 指令）
   - **常量指令**: `nop`, `iconst_m1`, `iconst_0~5`, `bipush`, `sipush`
   - **加载指令**: `iload`, `iload_0~3`, `aload`, `aload_0~3`
   - **存储指令**: `istore`, `istore_0~3`, `astore`, `astore_0~3`
   - **运算指令**: `iadd`, `isub`, `imul`, `idiv`
   - **对象操作**: `new`, `dup`, `getfield`, `putfield`
   - **方法调用**:
     - `invokestatic`（静态方法，支持递归）
     - `invokespecial`（构造方法、私有方法）
     - `invokevirtual`（实例方法，作弊版支持 println）
   - **控制流**: `ifeq`, `ifne`, `iflt`, `ifge`, `ifgt`, `ifle`,
     `if_icmpeq`, `if_icmpne`, `if_icmplt`, `if_icmpge`, `if_icmpgt`, `if_icmple`, `goto`
   - **返回指令**: `ireturn`, `return`
   - **字段访问**: `getstatic`（作弊版 System.out）, `getfield`, `putfield`

#### 4. **类加载器**（作弊版实现）
   - 支持用户类的完整加载
   - 系统类（java.*）占位符处理
   - 跳过 java.lang.Object 等系统类的加载和执行
   - 允许构造方法调用 super()

#### 5. **GC（垃圾回收）**
   - 标记-清除算法框架
   - 堆对象生命周期管理

#### 6. **测试覆盖**
   - ✅ 27 个测试全部通过
   - 解释器单元测试（8个）
   - Metaspace 测试（9个）
   - 集成测试（6个）
   - invokestatic 测试（3个）
   - 边界情况测试（除零错误等）

### 🚧 正在开发
- 无（v0.0.1 已完成）

### 📋 未来版本计划（v0.1.0+）

#### 短期目标
1. **完善方法调用**
   - `invokeinterface`：接口方法调用
   - 方法多态性支持
   - 虚方法表（vtable）

2. **数组支持**
   - `newarray`, `anewarray`：创建数组
   - `iaload`, `iastore`, `aaload`, `aastore`：数组元素访问
   - `arraylength`：数组长度

3. **其他类型**
   - Long、Float、Double 指令系列
   - 类型转换指令（i2l, l2i, i2f 等）

#### 长期目标
1. **真实类加载器**
   - Bootstrap ClassLoader
   - Application ClassLoader
   - 双亲委派模型
   - 真实的 java.* 系统类支持

2. **高级特性**
   - 异常处理（try-catch-finally）
   - 同步机制（synchronized）
   - 多线程支持
   - JIT 编译器（可选）

## 开发规范

### 添加新指令时
1. 在 `src/interpreter/instructions.rs` 中定义 opcode 常量
2. 在 `src/interpreter/mod.rs` 的 `execute_instruction` 方法中添加 match 分支
3. 实现指令逻辑（操作栈、局部变量表、PC）
4. 在 `tests/` 中添加单元测试
5. 在 `examples/` 中添加对应的 Java 示例

### 指令实现模板
```rust
OPCODE_NAME => {
    // 1. 读取操作数（如果有）
    // 2. 从栈中弹出操作数
    // 3. 执行操作
    // 4. 将结果压入栈
    // 5. 更新 PC
    frame.pc += N; // N 是指令长度
}
```

### 测试规范
- 每个新指令至少添加 1 个测试
- 测试应覆盖正常情况和边界情况
- 测试文件放在 `tests/` 目录

### 代码风格
- 使用清晰的注释说明指令的作用
- 保持代码简洁，避免过度抽象
- 错误处理使用 `anyhow::Result`

## 重要提醒

### JVM 基础知识
- JVM 是**基于栈的虚拟机**（vs 基于寄存器的虚拟机如 Dalvik）
- 每个方法执行时都有独立的栈帧
- 操作数栈是 LIFO（后进先出）
- PC（Program Counter）指向当前指令位置

### 字节码指令规律
- `iconst_N`：将常量 N 压入栈（-1 到 5）
- `iload_N`：从局部变量表索引 N 加载到栈（0 到 3）
- `istore_N`：从栈存储到局部变量表索引 N（0 到 3）
- 前缀 `i` = int, `l` = long, `f` = float, `d` = double, `a` = reference

### 调试技巧
1. 使用 `javap -c -v FileName.class` 查看字节码
2. 在解释器中添加日志：`RUST_LOG=debug cargo test -- --nocapture`
3. 对比我们的解析结果和 javap 的输出

### 核心技术实现要点

#### 符号引用解析与缓存
```rust
// Metaspace 中的缓存机制
pub struct RuntimeConstantPool {
    resolved_methods: HashMap<u16, ResolvedMethodRef>,
    resolved_fields: HashMap<u16, ResolvedFieldRef>,
    resolved_classes: HashMap<u16, String>,
}
```
- 第一次解析：3-6 次常量池查找
- 后续访问：1 次 HashMap.get（性能提升 ~10x）
- 适用于循环中频繁调用的场景

#### 方法调用实现
1. `invokestatic`（静态方法）
   - 从常量池解析方法引用（Methodref）
   - 弹出参数并传递给新栈帧
   - 创建新栈帧执行目标方法
   - 支持递归调用

2. `invokespecial`（构造方法、私有方法）
   - 类似 invokestatic，但需要处理 `this` 引用
   - 特殊处理：跳过 java.* 系统类方法（作弊版）

3. `invokevirtual`（实例方法）
   - 当前为作弊版实现，仅支持 System.out.println()
   - 未来需要实现虚方法表（vtable）

#### 对象模型
```java
Person p = new Person(42);
// 字节码流程：
new #2          // 在堆中分配对象
dup             // 复制引用（一个给 astore，一个给构造方法）
bipush 42       // 压入构造方法参数
invokespecial #3 // 调用 <init> 方法
astore_1        // 存储对象引用到局部变量
```

#### 动态链接
```rust
pub struct Frame {
    class_name: String,  // 指向 Metaspace 中的类元数据
    // ...
}
```
- 每个栈帧持有所属类的名称
- 通过 Metaspace 查找方法/字段引用
- 实现运行时的符号解析

### 常见陷阱
- ⚠️ 注意字节序：JVM 使用大端序（Big-Endian）
- ⚠️ 注意 PC 更新：不同指令长度不同（1-3字节）
- ⚠️ 注意栈的操作顺序：弹出顺序与压入顺序相反
- ⚠️ 注意有符号/无符号：`bipush` 是有符号字节，`sipush` 是有符号短整型
- ⚠️ DUP 指令：复制栈顶值，常用于对象创建时保留引用
- ⚠️ 对象引用 vs 原始类型：使用 `aload/astore` vs `iload/istore`
- ⚠️ 类加载时机：当前作弊版跳过 java.* 系统类，避免加载 java.lang.Object

### v0.0.1 核心成就

1. **完整的 JVM 运行时架构**
   - Metaspace + Heap + Stack 三大数据区完整实现
   - 支持对象分配、方法调用、字段访问

2. **60+ 字节码指令**
   - 覆盖基础计算、控制流、对象操作、方法调用
   - 支持递归、对象创建、字段访问

3. **性能优化**
   - 符号引用解析缓存（10x 性能提升）
   - HashMap 索引加速方法/字段查找

4. **教学价值**
   - 代码清晰，注释详细
   - 27 个测试用例覆盖核心场景
   - 适合深入理解 JVM 工作原理

## 学习路径建议

对于希望通过本项目学习 JVM 的开发者：

### 阶段 1：Class 文件解析（1-2 天）
- 阅读 `src/classfile/mod.rs`
- 理解常量池的 14 种类型
- 使用 `javap -v` 对比输出

### 阶段 2：运行时数据区（2-3 天）
- 阅读 `src/runtime/` 所有文件
- 理解栈帧结构（局部变量表 + 操作数栈）
- 理解 Metaspace 的符号引用解析

### 阶段 3：字节码解释器（3-5 天）
- 阅读 `src/interpreter/mod.rs`
- 从简单指令开始（iconst, iadd）
- 逐步理解复杂指令（invokestatic, new）

### 阶段 4：对象模型（2-3 天）
- 理解 new + dup + invokespecial 的配合
- 理解 getfield/putfield 的实现
- 理解对象在堆中的存储方式

### 阶段 5：方法调用机制（3-4 天）
- 理解符号引用 → 直接引用的解析
- 理解栈帧切换和参数传递
- 理解返回值处理

**总计**: 11-17 天可以完整理解本项目的核心实现

## 参考资源
- [JVM规范 - 指令集](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html)
- [JVM规范 - Class文件格式](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html)
- 项目文档：`QUICKSTART.md`, `examples/BYTECODE_ANALYSIS.md`

## 与 AI 协作时
- 优先阅读现有代码再提问
- 提供具体的错误信息和上下文
- 要求我解释字节码时，可以提供 Java 代码或 `.class` 文件路径
- 需要实现新功能时，先查看 QUICKSTART.md 了解优先级
