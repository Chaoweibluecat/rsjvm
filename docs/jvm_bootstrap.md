# JVM Bootstrap（启动加载）详解

## 核心问题：为什么main方法需要String？

```java
public static void main(String[] args)
```

这个签名意味着：
- ✅ 必须先加载 `java.lang.String` 类
- ✅ 必须先加载 `java.lang.Object` 类（String的父类）
- ✅ 可能还需要加载更多依赖类

**问题**：如果String还没加载，怎么调用main方法？

**答案**：JVM启动时会**预先加载**一批核心类（Bootstrap Classes）

## JVM启动的完整流程

### 第0步：JVM虚拟机初始化（C/C++代码）

```c
// hotspot/src/share/vm/runtime/thread.cpp
void Threads::create_vm(JavaVMInitArgs* args, bool* canTryAgain) {
    // 1. 初始化内存管理
    Universe::initialize_heap();

    // 2. 初始化类加载器
    SystemDictionary::initialize();

    // 3. 加载核心类（Bootstrap Classes）
    SystemDictionary::initialize_preloaded_classes();

    // 4. 初始化线程
    JavaThread::initialize_thread_locals();

    // 5. 准备执行Java代码
    // ...
}
```

### 第1步：加载Bootstrap Classes

```java
// JVM内部预加载的核心类（约200个）

// === 最核心的类（必须最先加载）===
java.lang.Object              // 所有类的父类
java.lang.Class               // 类的元类（描述类的类）
java.lang.String              // 字符串
java.lang.System              // 系统类

// === 基础类型包装类 ===
java.lang.Integer
java.lang.Long
java.lang.Boolean
// ... 其他包装类

// === 异常类 ===
java.lang.Throwable
java.lang.Exception
java.lang.RuntimeException
java.lang.Error
java.lang.NullPointerException
// ... 其他异常

// === 集合类（部分）===
java.util.ArrayList
java.util.HashMap
// ... 其他常用集合

// === 类加载器 ===
java.lang.ClassLoader
java.net.URLClassLoader

// === 线程相关 ===
java.lang.Thread
java.lang.ThreadGroup

// === 反射相关 ===
java.lang.reflect.Field
java.lang.reflect.Method
java.lang.reflect.Constructor

// === IO相关（部分）===
java.io.InputStream
java.io.OutputStream
java.io.PrintStream          // System.out 需要

// ... 总共约200个类
```

**为什么这么多？**
- String 依赖 Object, CharSequence, Comparable
- System 依赖 PrintStream, InputStream, OutputStream
- 异常处理需要 Throwable 及其子类
- 线程管理需要 Thread, ThreadGroup
- ...

### 第2步：初始化System类

```java
// java/lang/System.java
public final class System {
    // 标准输入/输出/错误流
    public final static InputStream in;
    public final static PrintStream out;
    public final static PrintStream err;

    // 静态初始化块（JVM启动时执行）
    static {
        initializeSystemClass();  // 本地方法
    }
}

// JVM在执行用户代码前会调用System的<clinit>方法
// 初始化 System.out, System.in, System.err
```

### 第3步：加载用户主类

```java
// 现在才加载用户的Hello类
ClassLoader.loadClass("Hello");

// 解析Hello.class
// 查找main方法
```

### 第4步：创建String[]参数

```java
// JVM创建命令行参数数组

// 步骤1：在堆上分配String数组对象
String[] args = new String[2];  // ← 需要String类已加载

// 步骤2：创建String对象
args[0] = new String("arg1");   // ← 需要String类已加载
args[1] = new String("arg2");

// 步骤3：传给main方法
Hello.main(args);
```

### 第5步：执行main方法

现在所有依赖都准备好了！

## 时间线图示

```
时间轴：
  0ms ┌─────────────────────────────────┐
      │ JVM进程启动（C/C++）              │
      │ - 初始化堆                       │
      │ - 初始化栈                       │
      │ - 初始化方法区                   │
      └─────────────────────────────────┘
  10ms ┌─────────────────────────────────┐
      │ 加载Bootstrap Classes            │
      │ - java.lang.Object    [1/200]   │
      │ - java.lang.Class     [2/200]   │
      │ - java.lang.String    [3/200]   │← main需要
      │ - java.lang.System    [4/200]   │
      │ - ... 继续加载 ...              │
      │ - java.io.PrintStream [50/200]  │
      │ - ... 继续加载 ...              │
      └─────────────────────────────────┘
  50ms ┌─────────────────────────────────┐
      │ 初始化System类                   │
      │ - 执行System.<clinit>            │
      │ - 初始化System.out               │
      │ - 初始化System.in                │
      │ - 初始化System.err               │
      └─────────────────────────────────┘
  60ms ┌─────────────────────────────────┐
      │ 加载用户主类 Hello.class         │
      │ - 读取字节码                     │
      │ - 解析class文件                  │
      │ - 验证字节码                     │
      └─────────────────────────────────┘
  65ms ┌─────────────────────────────────┐
      │ 查找main方法                     │
      └─────────────────────────────────┘
  66ms ┌─────────────────────────────────┐
      │ 创建String[] args                │← 使用已加载的String
      │ - 分配数组对象                   │
      │ - 创建String对象                 │
      └─────────────────────────────────┘
  70ms ┌─────────────────────────────────┐
      │ 执行Hello.main(args)             │← 开始执行用户代码
      │ - 创建栈帧                       │
      │ - 解释/编译字节码                │
      │ - ...                           │
      └─────────────────────────────────┘
```

## 验证：JVM启动时间

```bash
# 查看JVM启动时加载的类
java -verbose:class Hello 2>&1 | head -50
```

输出示例：
```
[Opened /Library/Java/.../rt.jar]
[Loaded java.lang.Object from /Library/Java/.../rt.jar]
[Loaded java.io.Serializable from /Library/Java/.../rt.jar]
[Loaded java.lang.Comparable from /Library/Java/.../rt.jar]
[Loaded java.lang.CharSequence from /Library/Java/.../rt.jar]
[Loaded java.lang.String from /Library/Java/.../rt.jar]    ← 这里！
[Loaded java.lang.reflect.AnnotatedElement from ...]
[Loaded java.lang.reflect.GenericDeclaration from ...]
[Loaded java.lang.reflect.Type from ...]
[Loaded java.lang.Class from /Library/Java/.../rt.jar]
[Loaded java.lang.Cloneable from /Library/Java/.../rt.jar]
[Loaded java.lang.ClassLoader from /Library/Java/.../rt.jar]
... (省略约200行)
[Loaded Hello from file:/Users/.../Hello.class]           ← 用户类
```

可以看到：String在用户类之前就加载了！

## String的依赖关系

```
java.lang.String 依赖：

java.lang.String
  ├─> java.lang.Object              (父类)
  ├─> java.lang.CharSequence         (接口)
  ├─> java.lang.Comparable<String>   (接口)
  ├─> java.io.Serializable           (接口)
  └─> char[]                         (内部存储)

这些都必须在String之前加载！
```

## 你的rsjvm怎么办？

### 选项1：不实现String支持（当前方案）

```rust
// 简化：main方法暂时不接受参数
pub fn execute_main(method: &MethodInfo) {
    // 忽略String[]参数
    // 只执行方法体
}
```

**优点**：
- ✅ 无需实现String类
- ✅ 无需实现对象
- ✅ 代码简单

**缺点**：
- ❌ 无法传递命令行参数
- ❌ 不能调用 `args.length`
- ❌ 不能使用字符串

### 选项2：Stub实现（模拟String）

```rust
// 创建一个假的String数组
pub struct FakeStringArray {
    args: Vec<String>,  // Rust String
}

// 当main方法需要args时，返回这个假对象
impl Interpreter {
    fn execute_main(&mut self, method: &MethodInfo, args: Vec<String>) {
        // 创建假的String[]
        let fake_args = FakeStringArray { args };

        // 压入局部变量表
        self.frame.set_local(0, JvmValue::FakeReference(fake_args));

        // 执行方法
        // ...
    }
}
```

**优点**：
- ✅ 可以接受命令行参数
- ✅ 可以模拟 `args.length`

**缺点**：
- ❌ 不是真正的JVM对象
- ❌ 无法调用String方法

### 选项3：完整实现（真实JVM方式）

```rust
pub struct JVM {
    // 需要实现：
    heap: Heap,                          // 堆
    method_area: MethodArea,             // 方法区
    bootstrap_classes: HashMap<String, ClassFile>,  // 预加载的类

    // String类
    string_class: ClassFile,
    // Object类
    object_class: ClassFile,
    // System类
    system_class: ClassFile,
}

impl JVM {
    pub fn bootstrap(&mut self) -> Result<()> {
        // 1. 加载Object类
        self.load_class("java/lang/Object")?;

        // 2. 加载String类
        self.load_class("java/lang/String")?;

        // 3. 加载System类
        self.load_class("java/lang/System")?;

        // ... 加载更多核心类

        Ok(())
    }

    pub fn create_string_array(&mut self, args: Vec<String>) -> usize {
        // 在堆上创建String[]数组
        let array_ref = self.heap.allocate_array("java/lang/String", args.len());

        // 为每个参数创建String对象
        for (i, arg) in args.iter().enumerate() {
            let string_ref = self.create_string(arg);
            self.heap.array_set(array_ref, i, string_ref);
        }

        array_ref
    }
}
```

**优点**：
- ✅ 完全符合JVM规范
- ✅ 可以正确处理所有String操作

**缺点**：
- ❌ 需要实现完整的类加载
- ❌ 需要实现堆和对象
- ❌ 需要提供rt.jar（或自己实现核心类）
- ❌ 工作量巨大（数千行代码）

## 实际的JVM实现

### OpenJDK的Bootstrap流程

```c
// hotspot/src/share/vm/classfile/systemDictionary.cpp

void SystemDictionary::initialize_preloaded_classes(TRAPS) {
    // 1. 加载最核心的类
    initialize_wk_klasses_through(WK_KLASS_ENUM_NAME(Object_klass),
                                   scan, CHECK);

    // 2. 加载String（依赖Object）
    initialize_wk_klasses_through(WK_KLASS_ENUM_NAME(String_klass),
                                   scan, CHECK);

    // 3. 加载Class（描述类的类）
    initialize_wk_klasses_through(WK_KLASS_ENUM_NAME(Class_klass),
                                   scan, CHECK);

    // 4. 加载其他核心类
    // ... 约200个类
}
```

### 预加载类的列表

位置：`hotspot/src/share/vm/classfile/vmSymbols.hpp`

```cpp
#define VM_SYMBOLS_DO(template, do_alias)                             \
  /* 核心类 */                                                         \
  template(java_lang_Object,                 "java/lang/Object")      \
  template(java_lang_Class,                  "java/lang/Class")       \
  template(java_lang_String,                 "java/lang/String")      \
  template(java_lang_Thread,                 "java/lang/Thread")      \
  template(java_lang_ThreadGroup,            "java/lang/ThreadGroup") \
  template(java_lang_Cloneable,              "java/lang/Cloneable")   \
  template(java_lang_Throwable,              "java/lang/Throwable")   \
  /* ... 继续约200个 ... */
```

## 结论

### 为什么main需要String？

1. ✅ JVM规范要求main的签名必须是 `(String[])`
2. ✅ 这样才能传递命令行参数
3. ✅ 保证跨平台一致性

### JVM如何解决这个问题？

1. ✅ 启动时预加载约200个核心类
2. ✅ 包括 Object, String, System 等
3. ✅ 总耗时约50-100ms（首次启动）

### 你的rsjvm建议？

**阶段1（当前）**：忽略参数
```rust
// main方法不处理args参数
// 只执行方法体
```

**阶段2（未来）**：Stub实现
```rust
// 创建假的String[]
// 足够演示功能
```

**阶段3（远期）**：完整实现
```rust
// 加载真实的java.lang.String
// 需要完整的类加载器
```

---

**你的观察非常正确**！String确实很特别，这是JVM设计的一个核心挑战。

查看完整文档：`docs/jvm_bootstrap.md`

要不要我帮你实现一个**简化版的main方法执行**（忽略String[]参数）？😊
