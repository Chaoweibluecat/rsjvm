# JVM完整实现路线图

## 问题：是否必须实现完整的内存模型和GC？

**简短回答**：看你的目标！

- **玩具解释器**（算术运算）：❌ 不需要
- **支持对象**（new Object）：✅ 需要堆 + 简单GC
- **生产级JVM**：✅ 需要完整内存模型 + 高级GC

## 当前rsjvm的状态

### ✅ 已实现（阶段1）

```rust
// src/interpreter/mod.rs
pub struct Interpreter {
    // 只有栈帧，无需堆
}

// 可以运行：
public static int add() {
    int a = 10;
    int b = 20;
    return a + b;  // ✅
}
```

### 🔨 框架已有（阶段2）

```rust
// src/runtime/heap.rs
pub struct Heap {
    objects: Vec<Option<Object>>,
    free_list: Vec<usize>,
}

// src/gc/mod.rs
pub struct GarbageCollector {
    roots: HashSet<usize>,
}
```

**缺少的连接**：
- ❌ 解释器还没使用堆
- ❌ 没有对象分配指令（`new`, `anewarray`）
- ❌ 没有字段访问指令（`getfield`, `putfield`）

## JVM内存模型完整版

### 1. 运行时数据区

```rust
pub struct JVM {
    // === 线程私有 ===
    // 每个线程一份

    /// PC寄存器（程序计数器）
    pc: usize,

    /// Java虚拟机栈
    stack: Vec<Frame>,

    /// 本地方法栈（JNI调用）
    native_stack: Vec<NativeFrame>,

    // === 线程共享 ===
    // 所有线程共享

    /// 堆（对象实例）
    heap: Heap,

    /// 方法区（类元数据）
    method_area: MethodArea,

    /// 运行时常量池
    runtime_constant_pool: HashMap<String, ConstantPool>,

    /// 直接内存（NIO）
    direct_memory: DirectMemory,
}
```

### 2. 堆的完整结构

```rust
pub struct Heap {
    // === 分代设计 ===

    /// 新生代（Young Generation）
    young_gen: YoungGeneration {
        eden: Region,       // Eden区（新对象分配）
        survivor0: Region,  // Survivor 0（GC后存活）
        survivor1: Region,  // Survivor 1（来回复制）
    },

    /// 老年代（Old Generation）
    old_gen: OldGeneration,

    /// 永久代（Java 7）/ 元空间（Java 8+）
    metaspace: Metaspace,

    // === 特殊区域 ===

    /// 字符串常量池
    string_pool: HashMap<String, ObjectRef>,

    /// 类静态变量
    static_fields: HashMap<String, JvmValue>,
}
```

### 3. 栈帧的完整结构

```rust
pub struct Frame {
    // === 当前实现 ===
    pub local_variables: Vec<JvmValue>,  // ✅ 已有
    pub operand_stack: Vec<JvmValue>,    // ✅ 已有

    // === 需要添加 ===

    /// 动态链接（指向运行时常量池）
    constant_pool_ref: &'static ConstantPool,

    /// 返回地址（方法返回后的PC）
    return_address: usize,

    /// 附加信息（调试、异常）
    additional_info: FrameInfo,
}
```

## 实现路线图

### 阶段1：当前状态 ✅

```
功能：
- ✅ 基础算术运算
- ✅ 局部变量
- ✅ 操作数栈

不需要：
- ❌ 堆
- ❌ GC
- ❌ 对象

示例：
int add(int a, int b) {
    return a + b;
}
```

### 阶段2：支持对象（需要堆 + 简单GC）

```rust
// 需要实现的指令
impl Interpreter {
    fn new_object(&mut self, class_name: &str) -> usize {
        // 1. 在堆上分配对象
        let obj_ref = self.heap.allocate(class_name);

        // 2. 压入操作数栈
        self.frame.push(JvmValue::Reference(obj_ref));

        // 3. 添加到GC根
        self.gc.add_root(obj_ref);

        obj_ref
    }

    fn get_field(&mut self, obj_ref: usize, field_name: &str) {
        // 从堆中获取对象
        let obj = self.heap.get(obj_ref).unwrap();
        let value = obj.fields.get(field_name).cloned();
        self.frame.push(value);
    }
}
```

**新增字节码支持**：
- `new` - 创建对象
- `getfield` - 读取字段
- `putfield` - 写入字段
- `anewarray` - 创建数组

**可以运行**：
```java
class Point {
    int x, y;
}

Point p = new Point();
p.x = 10;
p.y = 20;
```

### 阶段3：字符串支持（需要String池）

```rust
pub struct JVM {
    heap: Heap,
    string_pool: HashMap<String, usize>,  // String → ObjectRef
}

impl Interpreter {
    fn ldc_string(&mut self, string: &str) {
        // 检查String池
        if let Some(&obj_ref) = self.string_pool.get(string) {
            // 复用已有对象
            self.frame.push(JvmValue::Reference(obj_ref));
        } else {
            // 创建新String对象
            let obj_ref = self.heap.allocate("java/lang/String");
            // ... 初始化String对象
            self.string_pool.insert(string.to_string(), obj_ref);
            self.frame.push(JvmValue::Reference(obj_ref));
        }
    }
}
```

**可以运行**：
```java
String s1 = "Hello";
String s2 = "Hello";
System.out.println(s1 == s2);  // true（同一对象）
```

### 阶段4：垃圾回收（标记-清除）

```rust
impl GarbageCollector {
    pub fn collect(&mut self, heap: &mut Heap, stack: &[Frame]) -> usize {
        // 1. 确定GC Roots
        let roots = self.find_roots(stack);

        // 2. 标记阶段
        let reachable = self.mark(heap, &roots);

        // 3. 清除阶段
        let collected = self.sweep(heap, &reachable);

        collected
    }

    fn find_roots(&self, stack: &[Frame]) -> HashSet<usize> {
        let mut roots = HashSet::new();

        // 从栈中找所有对象引用
        for frame in stack {
            for value in &frame.local_variables {
                if let JvmValue::Reference(obj_ref) = value {
                    roots.insert(*obj_ref);
                }
            }
            for value in &frame.operand_stack {
                if let JvmValue::Reference(obj_ref) = value {
                    roots.insert(*obj_ref);
                }
            }
        }

        // 静态字段也是GC Root
        // ...

        roots
    }
}
```

### 阶段5：方法调用（需要方法区）

```rust
pub struct MethodArea {
    /// 已加载的类
    classes: HashMap<String, ClassInfo>,
}

pub struct ClassInfo {
    class_file: ClassFile,
    methods: HashMap<String, MethodInfo>,
    fields: HashMap<String, FieldInfo>,
    static_fields: HashMap<String, JvmValue>,
}

impl Interpreter {
    fn invoke_virtual(&mut self, method_name: &str, descriptor: &str) {
        // 1. 从操作数栈弹出对象引用
        let obj_ref = self.frame.pop_reference();

        // 2. 获取对象的实际类
        let obj = self.heap.get(obj_ref);
        let class_name = &obj.class_name;

        // 3. 从方法区查找方法
        let method = self.method_area.find_method(class_name, method_name);

        // 4. 创建新栈帧
        let new_frame = Frame::new(method.max_locals, method.max_stack);
        self.stack.push(new_frame);

        // 5. 执行方法字节码
        self.execute(&method.code);
    }
}
```

### 阶段6：异常处理

```rust
pub struct ExceptionHandler {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
}

impl Interpreter {
    fn execute_with_exception_handling(&mut self, code: &[u8]) -> Result<()> {
        for pc in 0..code.len() {
            match self.execute_instruction(code[pc]) {
                Ok(_) => continue,
                Err(exception) => {
                    // 查找异常处理器
                    if let Some(handler) = self.find_exception_handler(pc, &exception) {
                        self.pc = handler.handler_pc;
                        self.frame.push(JvmValue::Reference(exception));
                    } else {
                        // 没有处理器，向上传播
                        return Err(exception);
                    }
                }
            }
        }
        Ok(())
    }
}
```

### 阶段7：多线程支持

```rust
pub struct JVM {
    threads: Vec<Thread>,
    monitors: HashMap<usize, Monitor>,  // 对象锁
}

pub struct Thread {
    id: usize,
    stack: Vec<Frame>,
    pc: usize,
    status: ThreadStatus,
}

impl Interpreter {
    fn monitor_enter(&mut self, obj_ref: usize) {
        let monitor = self.monitors.entry(obj_ref).or_insert(Monitor::new());
        monitor.lock(self.thread_id);
    }

    fn monitor_exit(&mut self, obj_ref: usize) {
        let monitor = self.monitors.get_mut(&obj_ref).unwrap();
        monitor.unlock();
    }
}
```

## 最小可用JVM需要什么？

### 必须有的：

1. ✅ **堆**（Heap）
   - 对象分配
   - 对象访问

2. ✅ **GC**（Garbage Collector）
   - 标记-清除（最简单）
   - 或引用计数（更简单但有循环引用问题）

3. ✅ **方法区**（Method Area）
   - 存储类元数据
   - 方法查找

4. ✅ **栈**（Stack）
   - 已有！

5. ✅ **字符串池**（String Pool）
   - `ldc` 指令需要

### 可以暂时没有的：

- ❌ 分代GC（用简单标记-清除）
- ❌ JIT编译器（纯解释执行）
- ❌ 多线程（单线程）
- ❌ JNI（不调用本地代码）
- ❌ 反射（简化实现）

## 代码量估算

| 组件 | 当前行数 | 最小可用 | 生产级 |
|------|---------|---------|--------|
| **解释器** | ~200行 | ~1000行 | ~10000行 |
| **堆** | ~100行 | ~500行 | ~5000行 |
| **GC** | ~100行 | ~500行 | ~50000行 |
| **方法区** | 0行 | ~500行 | ~5000行 |
| **类加载器** | 框架 | ~1000行 | ~10000行 |
| **字节码** | 17条 | ~100条 | 200+条 |
| **总计** | ~2000行 | ~8000行 | ~500万行 |

## 现实建议

### 如果目标是学习：

```
阶段1（当前）：✅ 算术运算
  └─> 理解栈式虚拟机

阶段2：支持对象
  └─> 理解堆和GC

阶段3：方法调用
  └─> 理解调用栈

停在这里就够了！（~5000行代码）
```

### 如果目标是实用：

```
考虑基于现有JVM：
- GraalVM：用Java写JIT
- OpenJ9：IBM开源JVM
- 或直接用LLVM JIT

不要从零写生产级JVM（需要500万行代码）
```

## 你的下一步

### 选项A：添加对象支持（推荐）

1. 连接解释器和堆
2. 实现 `new` 指令
3. 实现 `getfield/putfield`
4. 触发GC（堆满时）

### 选项B：优化现有功能

1. 更多算术指令
2. 控制流（if/goto）
3. 方法调用（简化版）

### 选项C：研究OpenJDK

对比学习真实JVM实现：
```bash
# 克隆OpenJDK
git clone https://github.com/openjdk/jdk

# 查看HotSpot源码
cd jdk/src/hotspot/share
```

---

**结论**：

- ✅ 玩具JVM：不需要完整内存模型
- ✅ 支持对象：需要堆 + 简单GC（你已经有框架了！）
- ✅ 生产级：需要完整内存模型 + 高级GC（500万行代码）

**你的rsjvm**：
- 已完成阶段1
- 框架支持阶段2
- 距离"能运行对象"只差连接代码！

要不要我帮你实现 `new` 指令，连接堆和解释器？😊
