# 使用main方法 - rsjvm vs 真实JVM

## 🎉 新功能：自动查找并执行main方法

你的rsjvm现在可以像真实JVM一样自动查找并执行main方法了！

## 使用方法

### 方式1：自动查找main方法（新功能）

```bash
# 像真实JVM一样使用
./target/release/rsjvm run examples/SimpleMain.class

# 等价于真实JVM的：
java SimpleMain
```

### 方式2：手动指定方法（原有功能）

```bash
# 运行指定方法
./target/release/rsjvm run examples/ReturnOne.class --method returnOne

# 等价于真实JVM的：
# （真实JVM不支持这种用法，这是rsjvm的特色功能）
```

### 方式3：传递命令行参数（已解析但暂未实现）

```bash
# 命令行参数会被解析，但暂时无法传递给main方法
./target/release/rsjvm run examples/SimpleMain.class arg1 arg2

# 输出：
# 命令行参数: ["arg1", "arg2"] (注意：当前版本暂不支持传递参数)
```

## 功能对比

| 功能 | 真实JVM | rsjvm（当前） | 说明 |
|------|---------|--------------|------|
| **自动查找main** | ✅ | ✅ | 查找 `public static void main(String[])` |
| **执行main方法** | ✅ | ✅ | 执行字节码 |
| **命令行参数** | ✅ | ❌ | 需要String对象支持 |
| **方法调用** | ✅ | ❌ | 需要invokestatic指令 |
| **标准库** | ✅ | ❌ | 需要加载java.lang.* |

## 示例代码

### ✅ 可以运行的main方法

```java
// examples/SimpleMain.java
public class SimpleMain {
    public static void main(String[] args) {
        // ✅ 可以：算术运算
        int a = 10;
        int b = 20;
        int c = a + b;  // c = 30

        // ✅ 可以：局部变量
        int x = 100;
        int y = x * 2;

        // ✅ 可以：条件运算（如果实现了if指令）
        // int max = (a > b) ? a : b;
    }
}
```

编译并运行：
```bash
javac examples/SimpleMain.java
./target/release/rsjvm run examples/SimpleMain.class
```

输出：
```
正在加载: "examples/SimpleMain.class"

类名: SimpleMain
自动查找main方法...
✓ 找到main方法
方法签名: main : ([Ljava/lang/String;)V

=== 方法信息 ===
max_stack: 2
max_locals: 4
code_length: 11

字节码:
        0000  10 0a 3c 10 14 3d 1b 1c 60 3e b1

=== 开始执行 ===
✓ 执行成功！

方法无返回值 (void)
```

### ❌ 暂时不能运行的main方法

```java
public class AdvancedMain {
    public static void main(String[] args) {
        // ❌ 不可以：使用args参数
        int len = args.length;  // 需要String[]对象

        // ❌ 不可以：调用其他方法
        int result = calculate();  // 需要invokestatic指令

        // ❌ 不可以：使用标准库
        System.out.println("Hello");  // 需要加载java.lang.System
    }

    public static int calculate() {
        return 10 + 20;
    }
}
```

## main方法的要求

rsjvm会验证main方法必须满足：

1. ✅ 方法名必须是 `main`
2. ✅ 必须是 `public`
3. ✅ 必须是 `static`
4. ✅ 返回类型必须是 `void`
5. ✅ 参数必须是 `String[]`（描述符：`([Ljava/lang/String;)V`）

如果不满足任何一条，会报错：
```
Error: 找不到 public static void main(String[] args) 方法
```

## 查看class文件信息

使用parse命令查看class文件的详细信息：

```bash
./target/release/rsjvm parse examples/SimpleMain.class

# 输出包括：
# - 类名
# - 版本
# - 常量池
# - 方法列表（包括main）
```

详细模式（显示字节码）：
```bash
./target/release/rsjvm parse examples/SimpleMain.class -v
```

## 下一步改进

### 短期（1-2周）

1. **实现方法调用**
   - 实现 `invokestatic` 指令
   - 支持调用其他静态方法
   ```java
   public static void main(String[] args) {
       int result = calculate();  // ← 这样就可以了
   }
   ```

2. **实现控制流**
   - 实现 `if`, `goto` 指令
   - 支持条件判断和循环
   ```java
   public static void main(String[] args) {
       int x = 10;
       if (x > 5) {  // ← 这样就可以了
           x = x * 2;
       }
   }
   ```

### 中期（1-2月）

3. **支持对象创建**
   - 连接堆和解释器
   - 实现 `new`, `getfield`, `putfield` 指令
   ```java
   public static void main(String[] args) {
       Point p = new Point();  // ← 这样就可以了
       p.x = 10;
   }
   ```

4. **简化的String支持**
   - 实现String池
   - 支持 `ldc` 加载字符串
   ```java
   public static void main(String[] args) {
       String s = "Hello";  // ← 这样就可以了
   }
   ```

### 长期（3-6月）

5. **加载JDK核心类**
   - 实现类加载器
   - 支持加载 `java.lang.String`, `java.lang.Object` 等
   ```java
   public static void main(String[] args) {
       int len = args.length;  // ← 这样就可以了
       String s = args[0];
   }
   ```

6. **本地方法桥接**
   - 实现JNI接口
   - 桥接Rust函数到Java
   ```java
   public static void main(String[] args) {
       System.out.println("Hello");  // ← 这样就可以了
   }
   ```

## 对比：真实JVM的启动流程

### 真实JVM

```bash
java SimpleMain arg1 arg2
```

步骤：
1. 启动JVM（C/C++）
2. 加载Bootstrap Classes（~200个核心类）
3. 初始化System类
4. 加载 SimpleMain.class
5. 查找 `public static void main(String[])`
6. 创建 `String[] args = ["arg1", "arg2"]`
7. 执行 `SimpleMain.main(args)`

耗时：约70-100ms（首次启动）

### rsjvm（当前）

```bash
./target/release/rsjvm run examples/SimpleMain.class arg1 arg2
```

步骤：
1. 启动rsjvm（Rust）
2. 加载 SimpleMain.class
3. 查找 `public static void main(String[])`
4. 解析命令行参数（但暂不传递）
5. 执行 `SimpleMain.main(null)`

耗时：约1-5ms（极快！）

## 技术细节

### main方法的字节码特征

```
访问标志：0x0009 = ACC_PUBLIC (0x0001) | ACC_STATIC (0x0008)
方法名：main
描述符：([Ljava/lang/String;)V
        ^^^^^^^^^^^^^^^^^^^ ^
        参数：String[]      返回void
```

### 查找算法

```rust
fn find_main_method(class_file: &ClassFile) -> Result<&MethodInfo> {
    const ACC_PUBLIC: u16 = 0x0001;
    const ACC_STATIC: u16 = 0x0008;

    for method in &class_file.methods {
        let name = class_file.constant_pool.get_utf8(method.name_index)?;
        let descriptor = class_file.constant_pool.get_utf8(method.descriptor_index)?;

        if name == "main" && descriptor == "([Ljava/lang/String;)V" {
            if (method.access_flags & ACC_PUBLIC) != 0 &&
               (method.access_flags & ACC_STATIC) != 0 {
                return Ok(method);
            }
        }
    }

    Err(anyhow!("找不到main方法"))
}
```

## 常见问题

### Q: 为什么不支持命令行参数？

**A**: 需要实现String对象支持。当前版本专注于字节码执行，对象支持是下一阶段的目标。

### Q: 可以运行真实的Java程序吗？

**A**: 不能。rsjvm只能运行**不依赖标准库**的简单方法。真实程序需要：
- java.lang.* 核心类
- java.util.* 集合类
- java.io.* IO类
- ... 总共约3000个类

### Q: 与真实JVM的差距有多大？

**A**:
- 代码量：rsjvm ~2000行，OpenJDK ~500万行（差距2500倍）
- 字节码：rsjvm ~20条指令，JVM 200+条指令
- 功能：rsjvm ~1%，OpenJDK 100%

### Q: 学习价值在哪里？

**A**:
- ✅ 理解JVM工作原理
- ✅ 理解字节码执行过程
- ✅ 理解类文件结构
- ✅ 理解虚拟机设计思路
- ✅ Rust系统编程实践

---

**恭喜！** 你的rsjvm现在可以像真实JVM一样自动查找并执行main方法了！🎉
