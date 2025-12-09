# ILOAD 指令的访存实现详解

## 问题：`ILOAD_0` 到 `ILOAD_3` 的访存是怎么实现的？

### 核心代码

```rust
// src/interpreter/mod.rs:132
ILOAD_0 | ILOAD_1 | ILOAD_2 | ILOAD_3 => {
    let index = (opcode - ILOAD_0) as usize;  // 计算局部变量索引
    let value = frame.get_local(index)?.clone();  // 从局部变量表读取
    frame.push(value);  // 压入操作数栈
    frame.pc += 1;  // PC + 1
}
```

```rust
// src/runtime/frame.rs:52
pub fn get_local(&self, index: usize) -> Result<&JvmValue> {
    self.local_vars
        .get(index)  // Vec索引访问：O(1)
        .ok_or_else(|| anyhow!("Local variable index out of bounds: {}", index))
}
```

## 内存布局

### 栈帧结构

```
Frame {
    local_vars: Vec<JvmValue>,     // 局部变量表（固定大小）
    operand_stack: Vec<JvmValue>,  // 操作数栈（动态增长）
    pc: usize,                      // 程序计数器
}
```

### 内存示意图

```
栈帧内存布局：

┌─────────────────────────────────────┐
│ Frame                               │
├─────────────────────────────────────┤
│ local_vars: Vec<JvmValue>           │  ← 局部变量表
│   [0]: JvmValue::Int(10)            │  ← iload_0 读这里
│   [1]: JvmValue::Int(20)            │  ← iload_1 读这里
│   [2]: JvmValue::Int(0)             │  ← 未使用
│   [3]: JvmValue::Int(0)             │  ← 未使用
│   capacity: 4                       │
├─────────────────────────────────────┤
│ operand_stack: Vec<JvmValue>        │  ← 操作数栈
│   [栈底]                            │
│   ...                               │
│   [栈顶]                            │
│   capacity: 2                       │
├─────────────────────────────────────┤
│ pc: 6                               │  ← 当前指令位置
└─────────────────────────────────────┘

Rust Vec内部结构：
┌─────────────────┐
│ Vec<JvmValue>   │
├─────────────────┤
│ ptr: *mut T     │─┐  指向堆内存
│ len: 4          │ │
│ cap: 4          │ │
└─────────────────┘ │
                    │
                    ↓
    堆内存：         [JvmValue, JvmValue, JvmValue, JvmValue]
                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                     连续内存块（Rust Vec保证）
```

## 详细执行流程

### 示例：执行 `iload_0`

#### 步骤1：解释器读取指令

```rust
// execute_instruction 被调用
let opcode = code[frame.pc];  // 读取：opcode = 0x1A (ILOAD_0)
```

#### 步骤2：匹配指令

```rust
match opcode {
    ILOAD_0 | ILOAD_1 | ILOAD_2 | ILOAD_3 => {
        // opcode = 0x1A (ILOAD_0)
        // ILOAD_0 = 0x1A
```

#### 步骤3：计算索引

```rust
let index = (opcode - ILOAD_0) as usize;
// index = 0x1A - 0x1A = 0
```

**为什么这样计算？**

```
JVM指令编码（连续）：
ILOAD_0 = 0x1A
ILOAD_1 = 0x1B
ILOAD_2 = 0x1C
ILOAD_3 = 0x1D

计算索引：
0x1A - 0x1A = 0  ← iload_0
0x1B - 0x1A = 1  ← iload_1
0x1C - 0x1A = 2  ← iload_2
0x1D - 0x1A = 3  ← iload_3
```

#### 步骤4：从局部变量表读取

```rust
let value = frame.get_local(index)?.clone();
```

**调用栈**：
```
frame.get_local(0)
  ↓
self.local_vars.get(0)  // Rust Vec索引
  ↓
返回: &JvmValue::Int(10)
  ↓
.clone()  // 克隆值
  ↓
value = JvmValue::Int(10)
```

#### 步骤5：压入操作数栈

```rust
frame.push(value);
```

**内存变化**：
```
执行前：
operand_stack: []

执行后：
operand_stack: [JvmValue::Int(10)]
                ^^^^^^^^^^^^^^^^^^^
                栈顶
```

#### 步骤6：更新PC

```rust
frame.pc += 1;  // pc: 6 → 7
```

## 内存访问的实现细节

### 1. Rust Vec的内存布局

```rust
// local_vars: Vec<JvmValue>
// 内部实现（简化）：
pub struct Vec<T> {
    ptr: *mut T,      // 指向堆上的连续内存
    len: usize,       // 当前元素数量
    cap: usize,       // 容量
}
```

**内存示意**：
```
栈上（Vec结构）：
┌─────────────────┐
│ ptr: 0x1000     │───┐
│ len: 4          │   │
│ cap: 4          │   │
└─────────────────┘   │
                      │
堆上（实际数据）：     ↓
0x1000: [JvmValue::Int(10)]     ← local_vars[0]
0x1008: [JvmValue::Int(20)]     ← local_vars[1]
0x1010: [JvmValue::Int(0)]      ← local_vars[2]
0x1018: [JvmValue::Int(0)]      ← local_vars[3]
```

### 2. 索引访问的实现

```rust
// self.local_vars.get(index)
// 等价于：
fn get(&self, index: usize) -> Option<&T> {
    if index < self.len {
        // 计算地址：base_ptr + index * sizeof(T)
        let ptr = unsafe { self.ptr.add(index) };
        Some(unsafe { &*ptr })
    } else {
        None
    }
}
```

**性能**：O(1) 时间复杂度（数组索引）

### 3. JvmValue的大小

```rust
#[derive(Debug, Clone)]
pub enum JvmValue {
    Int(i32),              // 4字节 + tag
    Long(i64),             // 8字节 + tag
    Float(f32),            // 4字节 + tag
    Double(f64),           // 8字节 + tag
    Reference(Option<usize>), // 8字节 + tag
}

// Rust枚举的内存布局：
// sizeof(JvmValue) = max(所有variant的大小) + 判别标签
// = 8字节（最大variant）+ 1字节（tag）+ padding
// ≈ 16字节（对齐后）
```

**局部变量表的内存占用**：
```
max_locals = 4
内存占用 = 4 × 16字节 = 64字节（加上Vec开销）
```

## 不同的ILOAD指令对比

### ILOAD_0/1/2/3（单字节指令）

```
优点：
- 只占1个字节
- 常用局部变量（0-3）访问最快
- 无需额外参数

字节码：
1A        ← iload_0（1字节）
1B        ← iload_1（1字节）
1C        ← iload_2（1字节）
1D        ← iload_3（1字节）
```

### ILOAD（通用指令，2字节）

```
优点：
- 可以访问任意局部变量（0-255）
- 灵活

缺点：
- 占2个字节
- 需要读取参数

字节码：
15 04     ← iload 4（2字节）
15 0A     ← iload 10（2字节）
```

**实现**（如果要添加）：
```rust
ILOAD => {
    let index = code[frame.pc + 1] as usize;  // 读取下一个字节
    let value = frame.get_local(index)?.clone();
    frame.push(value);
    frame.pc += 2;  // PC + 2（指令+参数）
}
```

### WIDE ILOAD（3字节，支持0-65535）

```
极少使用，支持超大局部变量表

字节码：
C4 15 01 00    ← wide iload 256（4字节）
```

## 性能分析

### 访存时间复杂度

| 操作 | 时间复杂度 | 说明 |
|------|-----------|------|
| **计算索引** | O(1) | 简单减法 |
| **Vec索引** | O(1) | 数组访问 |
| **clone** | O(1) | 简单值拷贝 |
| **push** | O(1)* | 摊销常数时间 |
| **总计** | **O(1)** | 常数时间 |

*注：Vec的push在需要扩容时是O(n)，但摊销后是O(1)

### 与真实JVM的对比

| 特性 | rsjvm | HotSpot JVM |
|------|-------|-------------|
| **局部变量存储** | Rust Vec | 原生数组/寄存器 |
| **访问方式** | Vec索引 | 内存偏移/寄存器 |
| **JIT优化** | ❌ 无 | ✅ 寄存器分配 |
| **性能** | ~5ns/次 | ~1ns/次（JIT后）|

## 完整示例

### Java代码

```java
public static int test() {
    int a = 10;  // istore_0
    int b = 20;  // istore_1
    return a + b;  // iload_0, iload_1, iadd, ireturn
}
```

### 执行追踪

```
指令    PC   操作数栈         局部变量表
------  ---  --------------  ---------------
bipush  0    []               [0, 0]
        1    [10]             [0, 0]
istore  2    []               [10, 0]
bipush  3    []               [10, 0]
        4    [20]             [10, 0]
istore  5    []               [10, 20]
iload_0 6    []               [10, 20]  ← 这里！
        7    [10]             [10, 20]  ← 读取local_vars[0]
iload_1 7    [10]             [10, 20]  ← 这里！
        8    [10, 20]         [10, 20]  ← 读取local_vars[1]
iadd    8    [10, 20]         [10, 20]
        9    [30]             [10, 20]  ← 弹出两个值，压入结果
ireturn 9    [30]             [10, 20]
        返回 30
```

### 内存快照（iload_0执行时）

```
┌─────────────────────────────────┐
│ Frame                           │
├─────────────────────────────────┤
│ local_vars (Vec)                │
│   ptr: 0x1000 ───┐              │
│   len: 2         │              │
│   cap: 4         │              │
└──────────────────│──────────────┘
                   │
        堆内存     ↓
    0x1000: JvmValue::Int(10)  ← get_local(0) 返回这里
    0x1010: JvmValue::Int(20)  ← get_local(1) 返回这里
    0x1020: JvmValue::Int(0)
    0x1030: JvmValue::Int(0)

┌─────────────────────────────────┐
│ operand_stack (Vec)             │
│   ptr: 0x2000 ───┐              │
│   len: 0         │              │
│   cap: 2         │              │
└──────────────────│──────────────┘
                   │
        堆内存     ↓
    0x2000: [空]
    0x2010: [空]

执行 iload_0 后：
    0x2000: JvmValue::Int(10)  ← clone(local_vars[0])
```

## 总结

### ILOAD的实现本质

1. **指令编码巧妙**：`ILOAD_0` 到 `ILOAD_3` 是连续的opcode
2. **索引计算简单**：`index = opcode - ILOAD_0`
3. **访存高效**：直接Vec索引，O(1)时间
4. **类型安全**：Rust的类型系统保证内存安全

### 关键设计

- ✅ 用 `Vec<JvmValue>` 模拟局部变量表
- ✅ 用 `Vec<JvmValue>` 模拟操作数栈
- ✅ 用 `usize` 作为PC（程序计数器）
- ✅ 用 `enum JvmValue` 统一表示不同类型

### 优化空间

如果想提升性能：
1. 用固定大小数组代替Vec（避免堆分配）
2. 使用JIT编译（直接生成机器码）
3. 寄存器分配优化（常用变量直接用CPU寄存器）

但对于学习型JVM，当前实现已经很好了！
