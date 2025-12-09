# JVM执行引擎基础概念与核心指令

## 第一部分：执行引擎基础概念

### 1. JVM执行引擎是什么？

执行引擎是JVM的核心组件，负责**执行字节码指令**。

```
Java源码 (.java)
  ↓ javac编译
字节码 (.class)
  ↓ 类加载器
JVM内存 (加载到方法区)
  ↓ 执行引擎  ← 这里！
运行结果
```

**执行引擎的两种模式**：
1. **解释执行**：逐条解释字节码（你的rsjvm用的就是这个）
2. **编译执行**：JIT编译成机器码（HotSpot的优化）

### 2. 核心数据结构

#### 2.1 栈帧（Frame）

每个方法调用创建一个栈帧：

```rust
// src/runtime/frame.rs
pub struct Frame {
    local_vars: Vec<JvmValue>,      // 局部变量表
    operand_stack: Vec<JvmValue>,   // 操作数栈
    pc: usize,                       // 程序计数器
}
```

**内存布局**：
```
┌─────────────────────────────────┐
│ 栈帧 (Frame)                     │
├─────────────────────────────────┤
│ 1. 局部变量表 (Local Variables) │
│    [0]: 参数或局部变量            │
│    [1]: 局部变量                 │
│    [2]: 局部变量                 │
│    ...                          │
├─────────────────────────────────┤
│ 2. 操作数栈 (Operand Stack)      │
│    [栈底]                        │
│    ...                          │
│    [栈顶] ← 当前操作位置          │
├─────────────────────────────────┤
│ 3. 动态链接 (Dynamic Linking)    │
│    → 指向常量池                  │
├─────────────────────────────────┤
│ 4. 方法返回地址                  │
│    → 调用者的PC                  │
└─────────────────────────────────┘
```

#### 2.2 局部变量表

**作用**：存储方法参数和局部变量

```java
public static int calculate(int a, int b) {
    int c = a + b;
    int d = c * 2;
    return d;
}

// 局部变量表：
// [0]: a (参数)
// [1]: b (参数)
// [2]: c (局部变量)
// [3]: d (局部变量)
```

**特点**：
- ✅ 编译时确定大小（max_locals）
- ✅ 索引从0开始
- ✅ 静态方法：从0开始存参数
- ✅ 实例方法：[0]存this，参数从[1]开始

#### 2.3 操作数栈

**作用**：字节码指令的工作区

```java
int result = 10 + 20;

// 执行过程：
// 1. bipush 10      → 栈: [10]
// 2. bipush 20      → 栈: [10, 20]
// 3. iadd           → 栈: [30]
// 4. istore_0       → 栈: []，local[0] = 30
```

**特点**：
- ✅ 编译时确定最大深度（max_stack）
- ✅ LIFO（后进先出）
- ✅ 所有运算都在栈上进行

#### 2.4 程序计数器（PC）

**作用**：记录当前执行到哪条指令

```java
字节码：
  0: iconst_1       ← PC = 0
  1: istore_0       ← PC = 1
  2: iconst_2       ← PC = 2
  3: istore_1       ← PC = 3
  4: iload_0        ← PC = 4
  ...
```

### 3. JVM值类型

```rust
// src/runtime/frame.rs
#[derive(Debug, Clone)]
pub enum JvmValue {
    Int(i32),                    // 整数
    Long(i64),                   // 长整数
    Float(f32),                  // 浮点数
    Double(f64),                 // 双精度浮点数
    Reference(Option<usize>),    // 对象引用
}
```

**重要特性**：
- `Int`, `Float`: 占1个slot
- `Long`, `Double`: 占2个slot
- `Reference`: 对象的堆地址（索引）

## 第二部分：核心指令分类

JVM有200+条指令，分为以下几类：

### 1. 常量指令（Load Constant）

**作用**：将常量压入操作数栈

#### 1.1 iconst 系列（小整数）

```
iconst_m1  (0x02)  → push(-1)
iconst_0   (0x03)  → push(0)
iconst_1   (0x04)  → push(1)
iconst_2   (0x05)  → push(2)
iconst_3   (0x06)  → push(3)
iconst_4   (0x07)  → push(4)
iconst_5   (0x08)  → push(5)
```

**示例**：
```java
int a = 3;

// 字节码：
iconst_3   // 压栈：[3]
istore_0   // 存入local[0]，栈：[]
```

**实现**：
```rust
ICONST_3 => {
    frame.push(JvmValue::Int(3));
    frame.pc += 1;
}
```

#### 1.2 bipush（字节范围）

```
bipush <byte>  (0x10)  → push(byte)
范围：-128 到 127
```

**示例**：
```java
int x = 100;

// 字节码：
bipush 100   // 0x10 0x64
istore_0
```

**实现**：
```rust
BIPUSH => {
    let value = code[frame.pc + 1] as i8;  // 读取下一字节
    frame.push(JvmValue::Int(value as i32));
    frame.pc += 2;  // 跳过参数字节
}
```

#### 1.3 sipush（短整数范围）

```
sipush <short>  (0x11)  → push(short)
范围：-32768 到 32767
```

**示例**：
```java
int x = 10000;

// 字节码：
sipush 10000   // 0x11 0x27 0x10
istore_0
```

**实现**：
```rust
SIPUSH => {
    let value = i16::from_be_bytes([
        code[frame.pc + 1],
        code[frame.pc + 2]
    ]);
    frame.push(JvmValue::Int(value as i32));
    frame.pc += 3;
}
```

#### 1.4 ldc（从常量池加载）

```
ldc #index  (0x12)  → push(constant_pool[index])
用于：大整数、浮点数、字符串
```

**示例**：
```java
int x = 1000000;
String s = "Hello";

// 字节码：
ldc #7      // constant_pool[7] = Integer(1000000)
istore_0
ldc #8      // constant_pool[8] = String("Hello")
astore_1
```

### 2. 加载指令（Load）

**作用**：从局部变量表加载到操作数栈

#### 2.1 iload 系列（整数）

```
iload_0  (0x1A)  → push(local[0])
iload_1  (0x1B)  → push(local[1])
iload_2  (0x1C)  → push(local[2])
iload_3  (0x1D)  → push(local[3])
iload n  (0x15 n) → push(local[n])
```

**示例**：
```java
int a = 10;
int b = a;  // ← 需要加载a

// 字节码：
bipush 10
istore_0    // local[0] = 10
iload_0     // push(local[0]) → [10]
istore_1    // local[1] = pop() → local[1] = 10
```

**实现**：
```rust
ILOAD_0 | ILOAD_1 | ILOAD_2 | ILOAD_3 => {
    let index = (opcode - ILOAD_0) as usize;
    let value = frame.get_local(index)?.clone();
    frame.push(value);
    frame.pc += 1;
}
```

#### 2.2 其他类型的load

```
lload_n  → 加载long
fload_n  → 加载float
dload_n  → 加载double
aload_n  → 加载引用（对象）
```

### 3. 存储指令（Store）

**作用**：从操作数栈弹出，存入局部变量表

#### 3.1 istore 系列

```
istore_0  (0x3B)  → local[0] = pop()
istore_1  (0x3C)  → local[1] = pop()
istore_2  (0x3D)  → local[2] = pop()
istore_3  (0x3E)  → local[3] = pop()
```

**示例**：
```java
int a = 10;

// 字节码：
bipush 10    // 栈：[10]
istore_0     // local[0] = 10，栈：[]
```

**实现**：
```rust
ISTORE_0 | ISTORE_1 | ISTORE_2 | ISTORE_3 => {
    let index = (opcode - ISTORE_0) as usize;
    let value = frame.pop()?;
    frame.set_local(index, value)?;
    frame.pc += 1;
}
```

### 4. 运算指令（Arithmetic）

**作用**：对操作数栈的值进行运算

#### 4.1 加法

```
iadd  (0x60)  → int加法
ladd  (0x61)  → long加法
fadd  (0x62)  → float加法
dadd  (0x63)  → double加法
```

**示例**：
```java
int c = 10 + 20;

// 字节码：
bipush 10    // 栈：[10]
bipush 20    // 栈：[10, 20]
iadd         // 栈：[30]
istore_0     // local[0] = 30，栈：[]
```

**实现**：
```rust
IADD => {
    let v2 = frame.pop_int()?;  // 先弹出20
    let v1 = frame.pop_int()?;  // 再弹出10
    frame.push(JvmValue::Int(v1 + v2));  // 压入30
    frame.pc += 1;
}
```

#### 4.2 其他运算

```
减法：isub, lsub, fsub, dsub
乘法：imul, lmul, fmul, dmul
除法：idiv, ldiv, fdiv, ddiv
取模：irem, lrem, frem, drem
取负：ineg, lneg, fneg, dneg
```

**示例**：
```java
int a = 10;
int b = 3;
int c = a - b;  // 7
int d = a * b;  // 30
int e = a / b;  // 3
int f = a % b;  // 1
int g = -a;     // -10

// 字节码：
bipush 10
istore_0
bipush 3
istore_1

iload_0
iload_1
isub        // a - b
istore_2

iload_0
iload_1
imul        // a * b
istore_3

iload_0
iload_1
idiv        // a / b
istore 4

iload_0
iload_1
irem        // a % b
istore 5

iload_0
ineg        // -a
istore 6
```

### 5. 类型转换指令

```
i2l  → int转long
i2f  → int转float
i2d  → int转double
l2i  → long转int
f2i  → float转int
d2i  → double转int
```

**示例**：
```java
int a = 10;
long b = a;  // 需要类型转换

// 字节码：
bipush 10
istore_0
iload_0
i2l         // int → long
lstore_1
```

### 6. 比较指令

```
lcmp   → 比较long
fcmpl  → 比较float (NaN返回-1)
fcmpg  → 比较float (NaN返回1)
dcmpl  → 比较double (NaN返回-1)
dcmpg  → 比较double (NaN返回1)
```

**结果**：
- v1 > v2  → push(1)
- v1 == v2 → push(0)
- v1 < v2  → push(-1)

### 7. 控制转移指令

#### 7.1 条件跳转

```
ifeq <offset>     → if (pop() == 0) goto offset
ifne <offset>     → if (pop() != 0) goto offset
iflt <offset>     → if (pop() < 0) goto offset
ifge <offset>     → if (pop() >= 0) goto offset
ifgt <offset>     → if (pop() > 0) goto offset
ifle <offset>     → if (pop() <= 0) goto offset
```

**示例**：
```java
int x = 10;
if (x > 5) {
    x = 20;
}

// 字节码：
 0: bipush 10
 2: istore_0        // x = 10
 3: iload_0         // 加载x
 4: bipush 5
 6: if_icmple 12    // if x <= 5 goto 12
 9: bipush 20
11: istore_0        // x = 20
12: return          // 结束
```

#### 7.2 比较跳转

```
if_icmpeq <offset>  → if (v1 == v2) goto offset
if_icmpne <offset>  → if (v1 != v2) goto offset
if_icmplt <offset>  → if (v1 < v2) goto offset
if_icmpge <offset>  → if (v1 >= v2) goto offset
if_icmpgt <offset>  → if (v1 > v2) goto offset
if_icmple <offset>  → if (v1 <= v2) goto offset
```

#### 7.3 无条件跳转

```
goto <offset>  (0xA7)  → 跳转到offset
```

**示例**：循环
```java
int sum = 0;
for (int i = 0; i < 10; i++) {
    sum += i;
}

// 字节码（简化）：
 0: iconst_0
 1: istore_0        // sum = 0
 2: iconst_0
 3: istore_1        // i = 0
 4: iload_1         // 加载i
 5: bipush 10
 7: if_icmpge 18    // if i >= 10 goto 18
10: iload_0         // 加载sum
11: iload_1         // 加载i
12: iadd            // sum + i
13: istore_0        // sum = sum + i
14: iinc 1, 1       // i++
17: goto 4          // 回到循环开始
18: return
```

### 8. 方法调用指令

```
invokevirtual   → 调用实例方法（多态）
invokespecial   → 调用构造方法、私有方法、父类方法
invokestatic    → 调用静态方法
invokeinterface → 调用接口方法
invokedynamic   → 动态调用（lambda、方法引用）
```

**示例**：
```java
public static int add(int a, int b) {
    return a + b;
}

public static void main(String[] args) {
    int result = add(10, 20);
}

// main的字节码：
0: bipush 10
2: bipush 20
4: invokestatic #7  // Method add:(II)I
7: istore_0
8: return
```

### 9. 返回指令

```
ireturn  (0xAC)  → 返回int
lreturn  (0xAD)  → 返回long
freturn  (0xAE)  → 返回float
dreturn  (0xAF)  → 返回double
areturn  (0xB0)  → 返回对象引用
return   (0xB1)  → void返回
```

**示例**：
```java
public static int test() {
    return 42;
}

// 字节码：
0: bipush 42
2: ireturn     // 返回栈顶的42
```

**实现**：
```rust
IRETURN => {
    let return_value = frame.pop()?;
    return Ok(InstructionControl::Return(Some(return_value)));
}

RETURN => {
    return Ok(InstructionControl::Return(None));
}
```

### 10. 对象操作指令

```
new          → 创建对象
newarray     → 创建基本类型数组
anewarray    → 创建引用类型数组
arraylength  → 获取数组长度
getfield     → 获取对象字段
putfield     → 设置对象字段
getstatic    → 获取静态字段
putstatic    → 设置静态字段
```

### 11. 栈操作指令

```
pop      → 弹出栈顶1个slot
pop2     → 弹出栈顶2个slot
dup      → 复制栈顶1个slot
dup2     → 复制栈顶2个slot
swap     → 交换栈顶两个slot
```

## 第三部分：执行示例

### 示例1：简单计算

```java
public static int calculate() {
    int a = 10;
    int b = 20;
    return a + b;
}
```

**字节码**：
```
 0: bipush 10
 2: istore_0
 3: bipush 20
 5: istore_1
 6: iload_0
 7: iload_1
 8: iadd
 9: ireturn
```

**执行追踪**：
```
PC  指令        操作数栈        局部变量表      说明
0   bipush 10   []              [0, 0]
1               [10]            [0, 0]          压入10
2   istore_0    []              [10, 0]         存入local[0]
3   bipush 20   []              [10, 0]
4               [20]            [10, 0]         压入20
5   istore_1    []              [10, 20]        存入local[1]
6   iload_0     []              [10, 20]
7               [10]            [10, 20]        加载local[0]
7   iload_1     [10]            [10, 20]
8               [10, 20]        [10, 20]        加载local[1]
8   iadd        [10, 20]        [10, 20]
9               [30]            [10, 20]        计算10+20
9   ireturn     [30]            [10, 20]        返回30
```

### 示例2：条件判断

```java
public static int max(int a, int b) {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}
```

**字节码**：
```
 0: iload_0          // 加载a
 1: iload_1          // 加载b
 2: if_icmple 7      // if a <= b goto 7
 5: iload_0          // 加载a
 6: ireturn          // 返回a
 7: iload_1          // 加载b
 8: ireturn          // 返回b
```

**执行追踪**（假设a=30, b=20）：
```
PC  指令         操作数栈    局部变量表
0   iload_0      []          [30, 20]
1                [30]        [30, 20]
1   iload_1      [30]        [30, 20]
2                [30, 20]    [30, 20]
2   if_icmple 7  [30, 20]    [30, 20]  比较：30 > 20，不跳转
5   iload_0      []          [30, 20]
6                [30]        [30, 20]
6   ireturn                              返回30
```

### 示例3：循环

```java
public static int sum(int n) {
    int sum = 0;
    for (int i = 0; i < n; i++) {
        sum += i;
    }
    return sum;
}
```

**字节码**：
```
 0: iconst_0          // 0
 1: istore_1          // sum = 0
 2: iconst_0          // 0
 3: istore_2          // i = 0
 4: iload_2           // 加载i
 5: iload_0           // 加载n
 6: if_icmpge 17      // if i >= n goto 17
 9: iload_1           // 加载sum
10: iload_2           // 加载i
11: iadd              // sum + i
12: istore_1          // sum = sum + i
13: iinc 2, 1         // i++
16: goto 4            // 回到循环开始
17: iload_1           // 加载sum
18: ireturn           // 返回sum
```

## 第四部分：你的rsjvm已支持的指令

```rust
// src/interpreter/mod.rs

已实现（17条）：
✅ NOP          - 无操作
✅ ICONST_M1    - 压入-1
✅ ICONST_0-5   - 压入0-5
✅ BIPUSH       - 压入byte
✅ SIPUSH       - 压入short
✅ ILOAD_0-3    - 加载int
✅ ISTORE_0-3   - 存储int
✅ IADD         - int加法
✅ ISUB         - int减法
✅ IMUL         - int乘法
✅ IDIV         - int除法
✅ IRETURN      - 返回int
✅ RETURN       - void返回
```

## 第五部分：下一步可以实现的指令

### 优先级1：控制流（1周）

```rust
IF_ICMPEQ, IF_ICMPNE, IF_ICMPLT, IF_ICMPGE, IF_ICMPGT, IF_ICMPLE
IFEQ, IFNE, IFLT, IFGE, IFGT, IFLE
GOTO
```

有了这些，就能运行if和循环！

### 优先级2：方法调用（1周）

```rust
INVOKESTATIC    // 调用静态方法
```

有了这个，就能调用其他方法！

### 优先级3：数组支持（2周）

```rust
NEWARRAY       // 创建数组
IALOAD         // 加载数组元素
IASTORE        // 存储数组元素
ARRAYLENGTH    // 数组长度
```

### 优先级4：对象支持（3-4周）

```rust
NEW            // 创建对象
GETFIELD       // 读取字段
PUTFIELD       // 写入字段
```

需要连接堆和GC。

## 总结

### 核心概念

1. **栈帧**：方法调用的基本单位
2. **局部变量表**：存储参数和局部变量
3. **操作数栈**：所有运算的工作区
4. **程序计数器**：记录执行位置

### 指令分类

1. **常量** → 压栈
2. **加载** → 局部变量表→栈
3. **存储** → 栈→局部变量表
4. **运算** → 栈上计算
5. **控制** → 改变PC
6. **方法** → 调用其他方法
7. **返回** → 结束方法

### 执行流程

```
1. PC指向当前指令
2. 读取opcode
3. 执行指令（操作栈、局部变量表）
4. PC前进
5. 重复直到return
```

---

查看你的实现：
- `src/interpreter/mod.rs` - 解释器核心
- `src/runtime/frame.rs` - 栈帧实现
- `src/interpreter/instructions.rs` - 指令定义

下一步建议：实现控制流指令（if/goto），就能运行更复杂的程序了！
