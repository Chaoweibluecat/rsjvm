# RSJVM 运行示例指南

## 🎉 恭喜！你的JVM可以真正运行字节码了！

现在`rsjvm`不仅能解析class文件，还能**真正执行**字节码并返回结果！

---

## 快速开始

### 1. 编译Java文件

```bash
# 方式1: 单行表达式
echo 'public class Test { public static int add() { return 1 + 2; } }' > /tmp/Test.java
javac /tmp/Test.java

# 方式2: 使用examples目录中的文件
javac examples/Calculator.java
javac examples/ReturnOne.java
```

### 2. 运行字节码

```bash
# 运行默认方法 (add)
./target/release/rsjvm run /tmp/Test.class

# 指定方法名
./target/release/rsjvm run examples/ReturnOne.class --method calculate

# 查看详细执行过程
./target/release/rsjvm run examples/Calculator.class --method noOptimization
```

---

## 可运行的例子

### ✅ 例1：最简单的返回

```java
public static int returnOne() {
    return 1;
}
```

运行：
```bash
./target/release/rsjvm run examples/ReturnOne.class --method returnOne
```

输出：
```
=== 返回值 ===
int: 1
```

---

### ✅ 例2：局部变量和加法

```java
public static int addOne() {
    int a = 1;
    int b = 0;
    return a + b;
}
```

运行：
```bash
./target/release/rsjvm run examples/ReturnOne.class --method addOne
```

字节码：
```
04 3b     # iconst_1, istore_0  (a = 1)
03 3c     # iconst_0, istore_1  (b = 0)
1a 1b     # iload_0, iload_1    (加载 a, b)
60        # iadd                (a + b)
ac        # ireturn
```

输出：
```
=== 返回值 ===
int: 1
```

---

### ✅ 例3：复杂计算

```java
public static int calculate() {
    int a = 10;
    int b = 20;
    int c = a + b;
    return c;
}
```

运行：
```bash
./target/release/rsjvm run examples/ReturnOne.class --method calculate
```

输出：
```
=== 返回值 ===
int: 30
```

---

### ✅ 例4：多次运算

```java
public static int noOptimization() {
    int a = 10;
    int b = 20;
    int c = 30;
    return a + b + c;
}
```

运行：
```bash
./target/release/rsjvm run examples/Calculator.class --method noOptimization
```

字节码解析：
```
10 0a     # bipush 10
3b        # istore_0      (a = 10)
10 14     # bipush 20
3c        # istore_1      (b = 20)
10 1e     # bipush 30
3d        # istore_2      (c = 30)
1a        # iload_0       (加载 a)
1b        # iload_1       (加载 b)
60        # iadd          (a + b)
1c        # iload_2       (加载 c)
60        # iadd          ((a+b) + c)
ac        # ireturn       (返回 60)
```

输出：
```
=== 返回值 ===
int: 60
```

---

## 编译器优化观察

### 对比：优化 vs 无优化

```java
// 会被优化
public static int constantFolding() {
    return 10 + 20 + 30;  // 编译器算出 60
}

// 不会被优化
public static int noOptimization() {
    int a = 10;
    int b = 20;
    int c = 30;
    return a + b + c;     // 运行时计算
}
```

运行对比：

```bash
# 优化版本：只有3字节
./target/release/rsjvm run examples/Calculator.class --method constantFolding
# 字节码: 10 3c ac  (bipush 60, ireturn)

# 无优化版本：15字节
./target/release/rsjvm run examples/Calculator.class --method noOptimization
# 字节码: 10 0a 3b 10 14 3c 10 1e 3d 1a 1b 60 1c 60 ac
```

**学习要点**：
- Java编译器会在编译期计算常量表达式
- 使用局部变量会阻止编译期优化
- 这就是为什么要用`static final`常量

---

## 当前支持的操作

### ✅ 支持的指令类型

| 类型 | 指令 | 示例 |
|------|------|------|
| 常量 | iconst_m1~5, bipush, sipush | `return 5;` |
| 局部变量加载 | iload_0~3 | `int x = a;` |
| 局部变量存储 | istore_0~3 | `a = 10;` |
| 算术运算 | iadd, isub, imul, idiv | `a + b`, `a * b` |
| 方法返回 | ireturn, return | `return 42;` |

### ❌ 暂不支持

- 循环 (需要goto、if指令)
- 条件判断 (需要if_icmp*指令)
- 对象创建 (需要new指令)
- 方法调用 (需要invoke*指令)
- 数组 (需要*aload、*astore指令)

---

## 创建你自己的测试

### 模板

```java
public class MyTest {
    // 你的方法（必须是static，无参数或只有int参数）
    public static int myMethod() {
        // 只能用：int变量、四则运算、return
        int x = 10;
        int y = 20;
        return x * y + 5;
    }
}
```

编译并运行：
```bash
javac MyTest.java
./target/release/rsjvm run MyTest.class --method myMethod
```

---

## 调试技巧

### 1. 查看字节码

```bash
# 使用rsjvm
./target/release/rsjvm parse examples/Calculator.class -v

# 使用javap（官方工具）
javap -c -v examples/Calculator.class
```

### 2. 对比验证

```bash
# 手动计算期望值
# 比如 10 + 20 = 30

# 运行查看实际值
./target/release/rsjvm run examples/ReturnOne.class --method calculate
```

### 3. 修改Java代码实验

```java
// 实验1：改变操作数
int a = 100;  // 原来是10
int b = 200;  // 原来是20
return a + b; // 期望300

// 实验2：改变运算
return a - b; // 期望-100

// 实验3：嵌套运算
return (a + b) * 2; // 期望600
```

---

## 性能对比

### 字节码大小

| 代码 | 字节码大小 | 原因 |
|------|----------|------|
| `return 3;` | 2字节 | iconst_3, ireturn |
| `return 1+2;` | 2字节 | 编译器优化成iconst_3 |
| `int a=1; int b=2; return a+b;` | 8字节 | 无法优化 |

### 执行速度

理论上：
- 常量返回：2条指令
- 带变量计算：7条指令（多3.5倍）

实际在RSJVM中差别不大（都是纳秒级），但在生产JVM中：
- JIT会优化热点代码
- 但初始解释执行时有差别

---

## 常见问题

### Q: 为什么我的方法找不到？

A: 检查方法必须是：
- `public static`
- 返回类型是`int`（暂时）
- 方法名拼写正确（区分大小写）

```bash
# 错误
./target/release/rsjvm run Test.class --method Add  # 大小写错误

# 正确
./target/release/rsjvm run Test.class --method add
```

### Q: 为什么报错 "Unknown opcode"?

A: 你的方法使用了尚未实现的指令。检查是否用了：
- 循环 (`for`, `while`)
- 条件判断 (`if`)
- 方法调用
- 对象、数组

### Q: 返回值不对？

A: 检查：
1. Java的整数除法会截断：`5/2 = 2` (不是2.5)
2. 变量初始值：未初始化的局部变量默认是0
3. 运算顺序：`a + b * c` 先乘后加

### Q: 如何运行带参数的方法？

A: 暂时不支持。需要实现：
- 从命令行传参
- 将参数压入局部变量表
- 这是下一步的工作

---

## 下一步实验

### 实验1：实现新指令

在`src/interpreter/mod.rs`中添加：

```rust
IREM => {
    // 取模运算
    let v2 = frame.pop_int()?;
    let v1 = frame.pop_int()?;
    frame.push(JvmValue::Int(v1 % v2));
    frame.pc += 1;
}
```

然后测试：
```java
public static int modulo() {
    return 10 % 3;  // 应返回1
}
```

### 实验2：添加日志

修改解释器，打印每条指令的执行：

```rust
log::info!("PC={}, opcode=0x{:02x}", frame.pc, opcode);
```

运行时：
```bash
RUST_LOG=info ./target/release/rsjvm run Test.class
```

### 实验3：性能测试

创建复杂计算：
```java
public static int heavyCompute() {
    int result = 0;
    result = result + 1;
    result = result + 2;
    // ... 重复100次
    return result;
}
```

---

## 成就解锁 🏆

- ✅ 成功解析class文件
- ✅ 成功执行第一个字节码
- ✅ 正确返回计算结果
- ⏳ 实现循环支持
- ⏳ 实现条件判断
- ⏳ 实现方法调用

---

## 总结

恭喜！你现在有了一个**可以工作的字节码解释器**！

虽然功能还很基础，但已经足够理解：
- Java字节码的工作原理
- 栈式虚拟机的执行模型
- 编译器优化的效果
- JVM的内部结构

继续实现更多指令，你会对JVM有更深的理解！

---

**Happy Hacking! 🚀**
