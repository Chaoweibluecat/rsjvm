# JVM指令速查表

## 你的rsjvm已支持的指令 ✅

| 类别 | 指令 | opcode | 功能 | 示例 |
|------|------|--------|------|------|
| **常量** | `iconst_m1` | 0x02 | push(-1) | `int a = -1;` |
| | `iconst_0` | 0x03 | push(0) | `int a = 0;` |
| | `iconst_1` | 0x04 | push(1) | `int a = 1;` |
| | `iconst_2` | 0x05 | push(2) | `int a = 2;` |
| | `iconst_3` | 0x06 | push(3) | `int a = 3;` |
| | `iconst_4` | 0x07 | push(4) | `int a = 4;` |
| | `iconst_5` | 0x08 | push(5) | `int a = 5;` |
| | `bipush n` | 0x10 | push(n) | `int a = 100;` |
| | `sipush n` | 0x11 | push(n) | `int a = 10000;` |
| **加载** | `iload_0` | 0x1A | push(local[0]) | `int b = a;` |
| | `iload_1` | 0x1B | push(local[1]) | |
| | `iload_2` | 0x1C | push(local[2]) | |
| | `iload_3` | 0x1D | push(local[3]) | |
| **存储** | `istore_0` | 0x3B | local[0]=pop() | `a = 10;` |
| | `istore_1` | 0x3C | local[1]=pop() | |
| | `istore_2` | 0x3D | local[2]=pop() | |
| | `istore_3` | 0x3E | local[3]=pop() | |
| **运算** | `iadd` | 0x60 | v1+v2 | `c = a + b;` |
| | `isub` | 0x64 | v1-v2 | `c = a - b;` |
| | `imul` | 0x68 | v1*v2 | `c = a * b;` |
| | `idiv` | 0x6C | v1/v2 | `c = a / b;` |
| **返回** | `ireturn` | 0xAC | return int | `return 42;` |
| | `return` | 0xB1 | return void | `return;` |

## 下一步应该实现的指令 🎯

### 优先级1：控制流（让你能写if和循环）

| 指令 | opcode | 功能 | 示例 |
|------|--------|------|------|
| `ifeq` | 0x99 | if ==0 goto | `if (x == 0)` |
| `ifne` | 0x9A | if !=0 goto | `if (x != 0)` |
| `iflt` | 0x9B | if <0 goto | `if (x < 0)` |
| `ifge` | 0x9D | if >=0 goto | `if (x >= 0)` |
| `ifgt` | 0x9E | if >0 goto | `if (x > 0)` |
| `ifle` | 0x9F | if <=0 goto | `if (x <= 0)` |
| `if_icmpeq` | 0x9F | if v1==v2 goto | `if (a == b)` |
| `if_icmpne` | 0xA0 | if v1!=v2 goto | `if (a != b)` |
| `if_icmplt` | 0xA1 | if v1<v2 goto | `if (a < b)` |
| `if_icmpge` | 0xA2 | if v1>=v2 goto | `if (a >= b)` |
| `if_icmpgt` | 0xA3 | if v1>v2 goto | `if (a > b)` |
| `if_icmple` | 0xA4 | if v1<=v2 goto | `if (a <= b)` |
| `goto` | 0xA7 | 无条件跳转 | 循环 |

### 优先级2：方法调用

| 指令 | opcode | 功能 | 示例 |
|------|--------|------|------|
| `invokestatic` | 0xB8 | 调用静态方法 | `add(10, 20)` |

### 优先级3：更多运算

| 指令 | opcode | 功能 | 示例 |
|------|--------|------|------|
| `irem` | 0x70 | 取模 | `x % 3` |
| `ineg` | 0x74 | 取负 | `-x` |
| `ishl` | 0x78 | 左移 | `x << 2` |
| `ishr` | 0x7A | 右移 | `x >> 2` |
| `iand` | 0x7E | 按位与 | `x & 0xFF` |
| `ior` | 0x80 | 按位或 | `x \| y` |
| `ixor` | 0x82 | 按位异或 | `x ^ y` |
| `iinc` | 0x84 | 变量自增 | `i++` |

## 常用字节码模式

### 模式1：变量赋值

```java
int a = 10;
```
```
bipush 10
istore_0
```

### 模式2：算术运算

```java
int c = a + b;
```
```
iload_0    // a
iload_1    // b
iadd
istore_2   // c
```

### 模式3：条件判断（需要实现）

```java
if (a > b) {
    return a;
} else {
    return b;
}
```
```
iload_0           // a
iload_1           // b
if_icmple else    // if a <= b goto else
iload_0
ireturn
else:
iload_1
ireturn
```

### 模式4：for循环（需要实现）

```java
for (int i = 0; i < 10; i++) {
    sum += i;
}
```
```
iconst_0
istore_1     // i = 0
loop:
iload_1      // i
bipush 10
if_icmpge end // if i >= 10 goto end
iload_0      // sum
iload_1      // i
iadd
istore_0     // sum += i
iinc 1, 1    // i++
goto loop
end:
```

## 指令格式

```
单字节指令：
  opcode
  例：iadd (0x60)

带参数指令：
  opcode param1 [param2 ...]
  例：bipush 10 (0x10 0x0A)
      sipush 1000 (0x11 0x03 0xE8)
      goto offset (0xA7 0x00 0x0C)
```

## 记忆技巧

### 前缀含义

- `i` = int（整数）
- `l` = long（长整数）
- `f` = float（浮点）
- `d` = double（双精度）
- `a` = address/reference（引用）
- `b` = byte
- `s` = short

### 后缀含义

- `const` = 常量
- `load` = 加载
- `store` = 存储
- `add` = 加法
- `sub` = 减法
- `mul` = 乘法
- `div` = 除法
- `rem` = 取模
- `return` = 返回

### 数字后缀

- `_0`, `_1`, `_2`, `_3` = 快速访问局部变量0-3

---

**提示**：
- 查看完整教程：`docs/jvm_execution_engine.md`
- 查看实现：`src/interpreter/mod.rs`
- 测试你的代码：`cargo run -- run examples/SimpleMain.class`
