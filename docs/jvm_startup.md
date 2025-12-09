# JVM启动流程详解

## 命令行：`java Hello arg1 arg2`

### 阶段1：JVM初始化

```c
// JVM启动器（launcher）
int main(int argc, char** argv) {
    // 1. 解析命令行参数
    char* class_name = "Hello";
    char* args[] = {"arg1", "arg2"};

    // 2. 创建JVM实例
    JavaVM* jvm;
    JNIEnv* env;
    JavaVMInitArgs vm_args;

    // 3. 启动JVM
    JNI_CreateJavaVM(&jvm, &env, &vm_args);

    // 4. 加载主类
    jclass main_class = env->FindClass("Hello");

    // 5. 查找main方法
    jmethodID main_method = env->GetStaticMethodID(
        main_class,
        "main",                    // 方法名
        "([Ljava/lang/String;)V"   // 方法签名
    );

    // 6. 创建参数数组
    jobjectArray java_args = create_string_array(env, args, 2);

    // 7. 调用main方法
    env->CallStaticVoidMethod(main_class, main_method, java_args);

    // 8. 销毁JVM
    jvm->DestroyJavaVM();

    return 0;
}
```

### 阶段2：类加载（ClassLoader）

```java
// 步骤1：找到class文件
File file = new File("Hello.class");

// 步骤2：读取字节码
byte[] bytecode = Files.readAllBytes(file.toPath());

// 步骤3：解析class文件
ClassFile classFile = parseClassFile(bytecode);

// 步骤4：验证字节码
verifyBytecode(classFile);

// 步骤5：准备（分配内存）
prepareClass(classFile);

// 步骤6：解析（符号引用转直接引用）
resolveClass(classFile);

// 步骤7：初始化（执行<clinit>）
initializeClass(classFile);
```

### 阶段3：查找main方法

```java
// JVM内部查找main方法的逻辑

public Method findMainMethod(ClassFile classFile) {
    // 1. 遍历所有方法
    for (MethodInfo method : classFile.methods) {
        // 2. 获取方法名
        String name = constantPool.getUtf8(method.nameIndex);

        // 3. 获取方法描述符
        String descriptor = constantPool.getUtf8(method.descriptorIndex);

        // 4. 检查是否是main方法
        if (name.equals("main") &&
            descriptor.equals("([Ljava/lang/String;)V")) {

            // 5. 检查访问标志
            if ((method.accessFlags & ACC_PUBLIC) != 0 &&
                (method.accessFlags & ACC_STATIC) != 0) {
                return method;
            }
        }
    }

    // 找不到main方法
    throw new NoSuchMethodError("main");
}
```

**main方法的要求**：
- ✅ 必须是 `public`
- ✅ 必须是 `static`
- ✅ 返回类型 `void`
- ✅ 参数类型 `String[]`
- ✅ 方法名 `main`

### 阶段4：创建参数数组

```java
// JVM创建 String[] args

String[] args = new String[2];
args[0] = "arg1";
args[1] = "arg2";

// 内存布局：
// 堆：
//   [String数组对象]
//     length: 2
//     [0]: -> [String对象 "arg1"]
//     [1]: -> [String对象 "arg2"]
```

### 阶段5：执行main方法

```java
// JVM执行main方法

public void executeMain(Method mainMethod, String[] args) {
    // 1. 创建新的栈帧
    Frame frame = new Frame(
        mainMethod.maxLocals,  // 局部变量表大小
        mainMethod.maxStack    // 操作数栈大小
    );

    // 2. 设置参数（main方法只有一个参数：String[] args）
    frame.setLocal(0, new Reference(args));

    // 3. 压入虚拟机栈
    stack.push(frame);

    // 4. 获取方法字节码
    byte[] code = getMethodCode(mainMethod);

    // 5. 执行字节码
    interpret(code);

    // 6. 方法返回，弹出栈帧
    stack.pop();
}
```

## 完整流程图

```
命令行：java Hello arg1 arg2
    ↓
┌───────────────────────────────┐
│ 1. JVM启动器（C/C++）          │
│    - 解析命令行参数            │
│    - 创建JVM实例               │
└───────────────────────────────┘
    ↓
┌───────────────────────────────┐
│ 2. 类加载器                    │
│    - 查找 Hello.class          │
│    - 读取字节码                │
│    - 解析class文件             │
│    - 验证字节码                │
└───────────────────────────────┘
    ↓
┌───────────────────────────────┐
│ 3. 查找main方法                │
│    - 遍历方法列表              │
│    - 匹配方法签名：            │
│      public static void main   │
│      (String[] args)           │
└───────────────────────────────┘
    ↓
┌───────────────────────────────┐
│ 4. 准备参数                    │
│    - 创建 String[]             │
│    - 填充命令行参数            │
└───────────────────────────────┘
    ↓
┌───────────────────────────────┐
│ 5. 创建栈帧                    │
│    - 分配局部变量表            │
│    - 分配操作数栈              │
│    - 设置 args 参数            │
└───────────────────────────────┘
    ↓
┌───────────────────────────────┐
│ 6. 执行字节码                  │
│    - 解释器/JIT执行            │
│    - 调用其他方法              │
│    - 创建对象                  │
└───────────────────────────────┘
    ↓
┌───────────────────────────────┐
│ 7. 方法返回                    │
│    - 弹出栈帧                  │
│    - 执行finally块             │
│    - 销毁JVM                   │
└───────────────────────────────┘
    ↓
退出进程
```

## 你的rsjvm可以实现的简化版

### 当前实现（只能执行单个方法）

```rust
// 当前：手动指定方法
./target/release/rsjvm run examples/ReturnOne.class --method returnOne
```

### 建议改进：模拟真实JVM启动

```rust
// 新功能：像真实JVM一样启动
./target/release/rsjvm run examples/Hello.class arg1 arg2
                              ^^^^^^^^^^^^      ^^^^^^^^^
                              类文件            命令行参数
```

### 实现步骤

#### 1. 修改CLI参数解析

```rust
// src/main.rs
#[derive(Parser)]
enum Commands {
    /// 运行class文件（查找并执行main方法）
    Run {
        /// class文件路径
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// 命令行参数（传给main方法）
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}
```

#### 2. 实现main方法查找

```rust
// src/main.rs
fn find_main_method(class_file: &ClassFile) -> Result<&MethodInfo> {
    for method in &class_file.methods {
        let name = class_file.constant_pool.get_utf8(method.name_index)?;
        let descriptor = class_file.constant_pool.get_utf8(method.descriptor_index)?;

        // 检查是否是main方法
        if name == "main" && descriptor == "([Ljava/lang/String;)V" {
            // 检查访问标志：public static
            const ACC_PUBLIC: u16 = 0x0001;
            const ACC_STATIC: u16 = 0x0008;

            if (method.access_flags & ACC_PUBLIC) != 0 &&
               (method.access_flags & ACC_STATIC) != 0 {
                return Ok(method);
            }
        }
    }

    Err(anyhow!("找不到 public static void main(String[] args) 方法"))
}
```

#### 3. 执行main方法

```rust
fn run_main(class_file: &ClassFile, args: Vec<String>) -> Result<()> {
    println!("正在加载: {}", class_file.get_class_name()?);

    // 1. 查找main方法
    let main_method = find_main_method(class_file)?;
    println!("✓ 找到main方法");

    // 2. 获取方法的Code属性
    let code_attr = get_code_attribute(main_method, class_file)?;

    // 3. 创建解释器
    let mut interpreter = Interpreter::new();

    // 4. TODO: 创建String[]参数（需要对象支持）
    // 简化版：暂时忽略参数

    // 5. 执行main方法
    println!("=== 开始执行main方法 ===");
    interpreter.execute_method(
        &code_attr.code,
        code_attr.max_locals as usize,
        code_attr.max_stack as usize,
    )?;

    println!("✓ 程序执行完成");
    Ok(())
}
```

### 示例Java代码

```java
// examples/Hello.java
public class Hello {
    public static void main(String[] args) {
        // 简单版本（不依赖标准库）
        int result = calculate();
        // System.out.println(result);  // 需要标准库支持
    }

    public static int calculate() {
        int a = 10;
        int b = 20;
        return a + b;
    }
}
```

### 使用方式

```bash
# 编译Java
javac examples/Hello.java

# 像真实JVM一样运行
cargo run -- run examples/Hello.class
# 或
./target/release/rsjvm run examples/Hello.class

# 带参数（未来支持）
./target/release/rsjvm run examples/Hello.class arg1 arg2
```

## 真实JVM vs 你的rsjvm

| 功能 | 真实JVM | 你的rsjvm（当前） | 你的rsjvm（改进后） |
|------|---------|------------------|-------------------|
| **查找main方法** | ✅ 自动 | ❌ 手动指定 | ✅ 自动查找 |
| **命令行参数** | ✅ 传入 | ❌ 不支持 | ⚠️ 解析但暂不用 |
| **执行main** | ✅ 完整 | ⚠️ 执行字节码 | ✅ 执行main方法 |
| **标准库** | ✅ 完整 | ❌ 无 | ❌ 无（不影响学习）|

## 下一步建议

### 选项1：实现main方法查找（推荐）

**优点**：
- ✅ 更接近真实JVM
- ✅ 理解JVM启动流程
- ✅ 代码量小（~100行）

**缺点**：
- ❌ 暂时无法使用命令行参数（需要对象支持）

### 选项2：支持方法调用

实现 `invokestatic` 指令，让main能调用其他方法：

```java
public static void main(String[] args) {
    int result = calculate();  // ← 需要方法调用
}

public static int calculate() {
    return 10 + 20;
}
```

### 选项3：添加对象支持

实现堆和GC，支持创建String数组：

```java
public static void main(String[] args) {
    String name = args[0];  // ← 需要对象支持
}
```

---

**总结**：

你的理解完全正确！JVM：
1. ✅ 解析传入的第一个class文件（主类）
2. ✅ 查找 `public static void main(String[] args)` 方法
3. ✅ 创建参数数组
4. ✅ 执行main方法
5. ✅ 程序退出

要不要我帮你实现**自动查找并执行main方法**的功能？这样就更像真实的JVM了！😊
