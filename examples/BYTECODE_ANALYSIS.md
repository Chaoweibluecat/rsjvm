# 字节码分析文档

## 例1：PrintOne.java - `System.out.println(1)`

### Java源码
```java
public static void main(String[] args) {
    System.out.println(1);
}
```

### 字节码（十六进制）
```
b2 00 07 04 b6 00 0d b1
```

### 反汇编
```
0: getstatic     #7   // Field java/lang/System.out:Ljava/io/PrintStream;
3: iconst_1           // 将常数1压栈
4: invokevirtual #13  // Method java/io/PrintStream.println:(I)V
7: return
```

### 执行过程（操作数栈变化）

| PC | 指令 | 操作数栈 | 说明 |
|----|------|----------|------|
| 0 | getstatic #7 | [PrintStream] | 获取System.out对象引用 |
| 3 | iconst_1 | [PrintStream, 1] | 压入常数1 |
| 4 | invokevirtual #13 | [] | 调用println方法，弹出2个参数 |
| 7 | return | [] | 方法返回 |

### 常量池引用
- `#7` -> FieldRef -> `java/lang/System.out`
- `#13` -> MethodRef -> `java/io/PrintStream.println:(I)V`

---

## 例2：ReturnOne.java - 三个简单方法

### 方法1: `returnOne()` - 最简单

#### Java源码
```java
public static int returnOne() {
    return 1;
}
```

#### 字节码
```
04 ac
```

#### 反汇编
```
0: iconst_1    // 将常数1压栈
1: ireturn     // 返回栈顶int值
```

#### 执行过程
| PC | 指令 | 操作数栈 | 说明 |
|----|------|----------|------|
| 0 | iconst_1 | [1] | 压入常数1 |
| 1 | ireturn | [] | 弹出栈顶并返回 |

**这是当前解释器能运行的！**

---

### 方法2: `addOne()` - 局部变量

#### Java源码
```java
public static int addOne() {
    int a = 1;
    int b = 0;
    return a + b;
}
```

#### 字节码
```
04 3b 03 3c 1a 1b 60 ac
```

#### 反汇编
```
0: iconst_1    // 压入1
1: istore_0    // 存到局部变量0 (a = 1)
2: iconst_0    // 压入0
3: istore_1    // 存到局部变量1 (b = 0)
4: iload_0     // 加载局部变量0 (a)
5: iload_1     // 加载局部变量1 (b)
6: iadd        // 整数加法
7: ireturn     // 返回结果
```

#### 执行过程（详细）

| PC | 指令 | 操作数栈 | 局部变量表 [a, b] | 说明 |
|----|------|----------|------------------|------|
| 0 | iconst_1 | [1] | [?, ?] | 压入1 |
| 1 | istore_0 | [] | [1, ?] | 存储到a |
| 2 | iconst_0 | [0] | [1, ?] | 压入0 |
| 3 | istore_1 | [] | [1, 0] | 存储到b |
| 4 | iload_0 | [1] | [1, 0] | 加载a |
| 5 | iload_1 | [1, 0] | [1, 0] | 加载b |
| 6 | iadd | [1] | [1, 0] | 弹出两个值，压入和 |
| 7 | ireturn | [] | [1, 0] | 返回1 |

**这个也能运行！用到了局部变量表**

---

### 方法3: `calculate()` - 复杂计算

#### Java源码
```java
public static int calculate() {
    int a = 10;
    int b = 20;
    int c = a + b;
    return c;
}
```

#### 字节码
```
10 0a 3b 10 14 3c 1a 1b 60 3d 1c ac
```

#### 反汇编
```
0: bipush 10       // 将字节10压栈
2: istore_0        // 存到局部变量0 (a = 10)
3: bipush 20       // 将字节20压栈
5: istore_1        // 存到局部变量1 (b = 20)
6: iload_0         // 加载a
7: iload_1         // 加载b
8: iadd            // a + b
9: istore_2        // 存到局部变量2 (c = 30)
10: iload_2        // 加载c
11: ireturn        // 返回c
```

#### 执行过程

| PC | 指令 | 操作数栈 | 局部变量 [a, b, c] | 说明 |
|----|------|----------|--------------------|------|
| 0 | bipush 10 | [10] | [?, ?, ?] | 压入10 |
| 2 | istore_0 | [] | [10, ?, ?] | a = 10 |
| 3 | bipush 20 | [20] | [10, ?, ?] | 压入20 |
| 5 | istore_1 | [] | [10, 20, ?] | b = 20 |
| 6 | iload_0 | [10] | [10, 20, ?] | 加载a |
| 7 | iload_1 | [10, 20] | [10, 20, ?] | 加载b |
| 8 | iadd | [30] | [10, 20, ?] | 10+20=30 |
| 9 | istore_2 | [] | [10, 20, 30] | c = 30 |
| 10 | iload_2 | [30] | [10, 20, 30] | 加载c |
| 11 | ireturn | [] | [10, 20, 30] | 返回30 |

**这个也能运行！**

---

## 字节码指令速查表

### 常量指令
| 指令 | 操作码 | 说明 |
|------|--------|------|
| iconst_0 | 0x03 | 压入int常数0 |
| iconst_1 | 0x04 | 压入int常数1 |
| iconst_2 | 0x05 | 压入int常数2 |
| ... | ... | iconst_3/4/5类似 |
| bipush | 0x10 | 压入一个字节（-128~127） |
| sipush | 0x11 | 压入一个短整数（-32768~32767） |

### 局部变量操作
| 指令 | 操作码 | 说明 |
|------|--------|------|
| iload_0 | 0x1a | 加载局部变量0到栈 |
| iload_1 | 0x1b | 加载局部变量1到栈 |
| iload_2 | 0x1c | 加载局部变量2到栈 |
| istore_0 | 0x3b | 存储栈顶到局部变量0 |
| istore_1 | 0x3c | 存储栈顶到局部变量1 |
| istore_2 | 0x3d | 存储栈顶到局部变量2 |

### 运算指令
| 指令 | 操作码 | 说明 |
|------|--------|------|
| iadd | 0x60 | int加法 |
| isub | 0x64 | int减法 |
| imul | 0x68 | int乘法 |
| idiv | 0x6c | int除法 |

### 返回指令
| 指令 | 操作码 | 说明 |
|------|--------|------|
| ireturn | 0xac | 返回int |
| return | 0xb1 | void返回 |

### 对象/方法指令（目前未实现）
| 指令 | 操作码 | 说明 |
|------|--------|------|
| getstatic | 0xb2 | 获取静态字段 |
| invokevirtual | 0xb6 | 调用实例方法 |

---

## 对比C编译器

如果你在tiny-c编译器中生成了汇编代码：

### C代码
```c
int add() {
    int a = 1;
    int b = 0;
    return a + b;
}
```

### x86汇编（假设）
```asm
add:
    push rbp
    mov rbp, rsp
    mov DWORD PTR [rbp-4], 1   ; a = 1
    mov DWORD PTR [rbp-8], 0   ; b = 0
    mov eax, [rbp-4]           ; 加载a
    add eax, [rbp-8]           ; a + b
    pop rbp
    ret
```

### JVM字节码
```
iconst_1
istore_0
iconst_0
istore_1
iload_0
iload_1
iadd
ireturn
```

### 关键区别

| 特性 | C/x86 | JVM |
|------|-------|-----|
| 操作对象 | 寄存器/内存 | 虚拟操作数栈 |
| 局部变量 | 栈帧偏移[rbp-4] | 抽象索引[0] |
| 指令长度 | 可变（1-15字节） | 多为1字节+操作数 |
| 平台相关 | 是（x86） | 否（平台无关） |
| 调用约定 | 复杂（ABI） | 统一（描述符） |

---

## 下一步实验

### 1. 手动执行字节码
拿一张纸，模拟执行`addOne()`方法：
- 画出操作数栈和局部变量表
- 逐条指令执行
- 观察栈和变量表的变化

### 2. 修改Java代码
```java
public static int test() {
    int x = 5;
    int y = 3;
    return x * y;  // 观察乘法指令
}
```

编译后看字节码，找到`imul`指令。

### 3. 理解指令编码
为什么`iconst_1`是单字节`04`，而`bipush 10`是双字节`10 0a`？
- iconst系列：常用常数有专用指令（优化）
- bipush：需要一个操作数字节

### 4. 尝试运行解释器
修改`src/interpreter/mod.rs`，添加一个测试函数手动执行`returnOne()`的字节码。

---

## 学习建议

1. **先手动执行**：用纸笔模拟3-5个简单方法
2. **对比汇编**：思考JVM指令和x86指令的区别
3. **实现解释器**：在`interpreter`模块中实现这些简单指令的执行
4. **逐步复杂化**：从纯计算 -> 对象 -> 方法调用

**你现在的优势**：有编译器经验，理解指令、栈帧、代码生成，学JVM字节码会很快！
