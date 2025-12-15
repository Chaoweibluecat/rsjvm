//! # 字节码指令定义
//!
//! JVM有超过200条字节码指令，这里定义了最常用的一部分。
//!
//! ## 指令格式
//! 每条指令由一个字节的操作码（opcode）和可选的操作数组成。
//!
//! ## 命名规则
//! - i: int (32位整数)
//! - l: long (64位长整数)
//! - f: float (32位浮点数)
//! - d: double (64位双精度浮点数)
//! - a: reference (对象引用/地址)
//!
//! JVM解释器模型
//! 1.Operand Stack: 操作数栈
//! 2.Local Variable Table: 局部变量表
//! 3.PC
//! 4.RA
//! 5.常量池的指针, 用来解析一些立即数对应的常量
//! 和机器码(操作数和结果都在寄存器上)不同, JVM是栈式的（操作数和结果都在栈顶）, 读到一个运算指令时, 则对操作数栈顶元素作为操作数操作
//! JVM 指令
//!  核心准则：
//! 1. Load : 局部变量表 -> operand stack, Store : operand stack -> 局部变量表
//! 2. XConst, Push: Immediate 压栈, LDC, 常量池压栈
//! 3. 运算指令
//! 4. 控制指令, 待补充

/// 字节码操作码常量
pub mod opcodes {
    // ============ 常量指令 (Constants) ============
    // 这些指令用于将常量值压入操作数栈

    /// 0x00 - 什么都不做，用于对齐或占位
    pub const NOP: u8 = 0x00;

    /// 0x01 - 将null引用压入栈（用于对象引用）
    pub const ACONST_NULL: u8 = 0x01;

    /// 0x02 - 将int常量-1压入栈
    pub const ICONST_M1: u8 = 0x02;

    /// 0x03 - 将int常量0压入栈
    /// 示例: int x = 0; 编译后使用iconst_0
    pub const ICONST_0: u8 = 0x03;

    /// 0x04 - 将int常量1压入栈
    pub const ICONST_1: u8 = 0x04;

    /// 0x05 - 将int常量2压入栈
    pub const ICONST_2: u8 = 0x05;

    /// 0x06 - 将int常量3压入栈
    pub const ICONST_3: u8 = 0x06;

    /// 0x07 - 将int常量4压入栈
    pub const ICONST_4: u8 = 0x07;

    /// 0x08 - 将int常量5压入栈
    /// 注：-1到5这些常量很常用，所以有专门的指令，无需额外操作数
    pub const ICONST_5: u8 = 0x08;

    /// 0x09 - 将long常量0L压入栈（占用2个栈槽）
    pub const LCONST_0: u8 = 0x09;
    /// 0x0a - 将long常量1L压入栈
    pub const LCONST_1: u8 = 0x0a;

    /// 0x0b - 将float常量0.0f压入栈
    pub const FCONST_0: u8 = 0x0b;
    /// 0x0c - 将float常量1.0f压入栈
    pub const FCONST_1: u8 = 0x0c;
    /// 0x0d - 将float常量2.0f压入栈
    pub const FCONST_2: u8 = 0x0d;

    /// 0x0e - 将double常量0.0压入栈（占用2个栈槽）
    pub const DCONST_0: u8 = 0x0e;
    /// 0x0f - 将double常量1.0压入栈
    pub const DCONST_1: u8 = 0x0f;

    /// 0x10 - 将一个byte值扩展为int并压入栈
    /// 格式: bipush <byte>
    /// 示例: int x = 100; 使用bipush 100
    pub const BIPUSH: u8 = 0x10;

    /// 0x11 - 将一个short值扩展为int并压入栈
    /// 格式: sipush <byte1> <byte2>
    /// 用于-128到127之外的小整数
    pub const SIPUSH: u8 = 0x11;

    /// 0x12 - 从常量池加载int/float/String常量（索引为1字节）
    /// 格式: ldc <index>
    /// 这是加载常量池数据的核心指令
    pub const LDC: u8 = 0x12;

    /// 0x13 - 从常量池加载常量（索引为2字节，宽索引版本）
    pub const LDC_W: u8 = 0x13;

    /// 0x14 - 从常量池加载long/double常量（占2个栈槽）
    pub const LDC2_W: u8 = 0x14;

    // ============ 加载指令 (Load) ============
    // 从局部变量表加载值到操作数栈
    // 局部变量表：每个方法的栈帧都有一个局部变量数组，用于存储方法参数和局部变量

    /// 0x15 - 从局部变量表加载int到栈
    /// 格式: iload <index>
    /// 示例: int x = 5; int y = x; // y=x时使用iload加载x的值
    pub const ILOAD: u8 = 0x15;
    /// 0x16 - 从局部变量表加载long到栈
    pub const LLOAD: u8 = 0x16;
    /// 0x17 - 从局部变量表加载float到栈
    pub const FLOAD: u8 = 0x17;
    /// 0x18 - 从局部变量表加载double到栈
    pub const DLOAD: u8 = 0x18;
    /// 0x19 - 从局部变量表加载引用到栈
    pub const ALOAD: u8 = 0x19;

    /// 0x1a - 加载局部变量表索引0的int（常用优化，省略索引参数）
    /// 注：实例方法中索引0是this引用，索引1开始是方法参数
    pub const ILOAD_0: u8 = 0x1a;
    /// 0x1b - 加载局部变量表索引1的int
    pub const ILOAD_1: u8 = 0x1b;
    /// 0x1c - 加载局部变量表索引2的int
    pub const ILOAD_2: u8 = 0x1c;
    /// 0x1d - 加载局部变量表索引3的int
    pub const ILOAD_3: u8 = 0x1d;

    /// 0x1e - 加载局部变量表索引0的long
    pub const LLOAD_0: u8 = 0x1e;
    pub const LLOAD_1: u8 = 0x1f;
    pub const LLOAD_2: u8 = 0x20;
    pub const LLOAD_3: u8 = 0x21;

    pub const FLOAD_0: u8 = 0x22;
    pub const FLOAD_1: u8 = 0x23;
    pub const FLOAD_2: u8 = 0x24;
    pub const FLOAD_3: u8 = 0x25;

    pub const DLOAD_0: u8 = 0x26;
    pub const DLOAD_1: u8 = 0x27;
    pub const DLOAD_2: u8 = 0x28;
    pub const DLOAD_3: u8 = 0x29;

    /// 0x2a - 加载局部变量表索引0的引用（实例方法中通常是this）
    pub const ALOAD_0: u8 = 0x2a;
    pub const ALOAD_1: u8 = 0x2b;
    pub const ALOAD_2: u8 = 0x2c;
    pub const ALOAD_3: u8 = 0x2d;

    // ============ 数组加载指令 (Array Load) ============
    // 从数组中加载元素到栈
    // 执行过程：栈顶是索引index，下面是数组引用arrayref
    // 弹出这两个值，从数组中取出arrayref[index]，压入栈

    /// 0x2e - 从int数组加载元素
    /// 栈变化: ..., arrayref, index → ..., value
    pub const IALOAD: u8 = 0x2e;
    /// 0x2f - 从long数组加载元素
    pub const LALOAD: u8 = 0x2f;
    /// 0x30 - 从float数组加载元素
    pub const FALOAD: u8 = 0x30;
    /// 0x31 - 从double数组加载元素
    pub const DALOAD: u8 = 0x31;
    /// 0x32 - 从引用数组加载元素
    pub const AALOAD: u8 = 0x32;
    /// 0x33 - 从byte/boolean数组加载元素
    pub const BALOAD: u8 = 0x33;
    /// 0x34 - 从char数组加载元素
    pub const CALOAD: u8 = 0x34;
    /// 0x35 - 从short数组加载元素
    pub const SALOAD: u8 = 0x35;

    // ============ 存储指令 (Store) ============
    // 从操作数栈顶弹出值，存储到局部变量表
    // 与加载指令相反：load是从局部变量表→栈，store是从栈→局部变量表

    /// 0x36 - 将栈顶int值存储到局部变量表
    /// 格式: istore <index>
    /// 示例: int x = 5; // 常量5先压栈，然后istore存入x的位置
    pub const ISTORE: u8 = 0x36;
    /// 0x37 - 将栈顶long值存储到局部变量表
    pub const LSTORE: u8 = 0x37;
    /// 0x38 - 将栈顶float值存储到局部变量表
    pub const FSTORE: u8 = 0x38;
    /// 0x39 - 将栈顶double值存储到局部变量表
    pub const DSTORE: u8 = 0x39;
    /// 0x3a - 将栈顶引用值存储到局部变量表
    pub const ASTORE: u8 = 0x3a;

    /// 0x3b - 存储int到局部变量表索引0（优化版本，无需索引参数）
    pub const ISTORE_0: u8 = 0x3b;
    pub const ISTORE_1: u8 = 0x3c;
    pub const ISTORE_2: u8 = 0x3d;
    pub const ISTORE_3: u8 = 0x3e;

    pub const LSTORE_0: u8 = 0x3f;
    pub const LSTORE_1: u8 = 0x40;
    pub const LSTORE_2: u8 = 0x41;
    pub const LSTORE_3: u8 = 0x42;

    pub const FSTORE_0: u8 = 0x43;
    pub const FSTORE_1: u8 = 0x44;
    pub const FSTORE_2: u8 = 0x45;
    pub const FSTORE_3: u8 = 0x46;

    pub const DSTORE_0: u8 = 0x47;
    pub const DSTORE_1: u8 = 0x48;
    pub const DSTORE_2: u8 = 0x49;
    pub const DSTORE_3: u8 = 0x4a;

    pub const ASTORE_0: u8 = 0x4b;
    pub const ASTORE_1: u8 = 0x4c;
    pub const ASTORE_2: u8 = 0x4d;
    pub const ASTORE_3: u8 = 0x4e;

    // ============ 栈操作指令 (Stack) ============
    // 直接操作操作数栈，不涉及局部变量表

    /// 0x57 - 弹出栈顶的一个单字值（int/float/reference）
    /// 栈变化: ..., value → ...
    pub const POP: u8 = 0x57;

    /// 0x58 - 弹出栈顶的一个双字值（long/double）或两个单字值
    pub const POP2: u8 = 0x58;

    /// 0x59 - 复制栈顶值
    /// 栈变化: ..., value → ..., value, value
    /// 示例: x = y = 5; 需要将5复制一份分别赋给x和y
    pub const DUP: u8 = 0x59;

    /// 0x5a - 复制栈顶值并插入到第二个值下面
    /// 栈变化: ..., value2, value1 → ..., value1, value2, value1
    pub const DUP_X1: u8 = 0x5a;

    /// 0x5b - 复制栈顶值并插入到第三个值下面
    pub const DUP_X2: u8 = 0x5b;

    /// 0x5c - 复制栈顶两个值
    pub const DUP2: u8 = 0x5c;

    pub const DUP2_X1: u8 = 0x5d;
    pub const DUP2_X2: u8 = 0x5e;

    /// 0x5f - 交换栈顶两个值
    /// 栈变化: ..., value2, value1 → ..., value1, value2
    pub const SWAP: u8 = 0x5f;

    // ============ 算术指令 (Arithmetic) ============
    // JVM支持int、long、float、double四种类型的算术运算
    // 所有运算都是从栈顶弹出操作数，计算后压回结果

    /// 0x60 - int加法
    /// 栈变化: ..., value1, value2 → ..., result
    /// 示例: int c = a + b; 对应指令序列 iload_1, iload_2, iadd, istore_3
    pub const IADD: u8 = 0x60;
    /// 0x61 - long加法
    pub const LADD: u8 = 0x61;
    /// 0x62 - float加法
    pub const FADD: u8 = 0x62;
    /// 0x63 - double加法
    pub const DADD: u8 = 0x63;

    /// 0x64 - int减法
    /// 栈变化: ..., value1, value2 → ..., result (result = value1 - value2)
    pub const ISUB: u8 = 0x64;
    pub const LSUB: u8 = 0x65;
    pub const FSUB: u8 = 0x66;
    pub const DSUB: u8 = 0x67;

    /// 0x68 - int乘法
    pub const IMUL: u8 = 0x68;
    pub const LMUL: u8 = 0x69;
    pub const FMUL: u8 = 0x6a;
    pub const DMUL: u8 = 0x6b;

    /// 0x6c - int除法（注意：除以0会抛出ArithmeticException）
    pub const IDIV: u8 = 0x6c;
    pub const LDIV: u8 = 0x6d;
    pub const FDIV: u8 = 0x6e;
    pub const DDIV: u8 = 0x6f;

    /// 0x70 - int取余（模运算）
    pub const IREM: u8 = 0x70;
    pub const LREM: u8 = 0x71;
    pub const FREM: u8 = 0x72;
    pub const DREM: u8 = 0x73;

    /// 0x74 - int取负（一元运算）
    /// 栈变化: ..., value → ..., -value
    pub const INEG: u8 = 0x74;
    pub const LNEG: u8 = 0x75;
    pub const FNEG: u8 = 0x76;
    pub const DNEG: u8 = 0x77;

    // ============ 位运算指令 (Bitwise) ============
    // 仅支持int和long类型

    /// 0x78 - int左移 (<<)
    /// 栈变化: ..., value, shift → ..., result
    pub const ISHL: u8 = 0x78;
    pub const LSHL: u8 = 0x79;

    /// 0x7a - int算术右移 (>>)，符号位扩展
    pub const ISHR: u8 = 0x7a;
    pub const LSHR: u8 = 0x7b;

    /// 0x7c - int逻辑右移 (>>>)，零扩展
    pub const IUSHR: u8 = 0x7c;
    pub const LUSHR: u8 = 0x7d;

    /// 0x7e - int按位与 (&)
    pub const IAND: u8 = 0x7e;
    pub const LAND: u8 = 0x7f;

    /// 0x80 - int按位或 (|)
    pub const IOR: u8 = 0x80;
    pub const LOR: u8 = 0x81;

    /// 0x82 - int按位异或 (^)
    pub const IXOR: u8 = 0x82;
    pub const LXOR: u8 = 0x83;

    /// 0x84 - int增量（直接操作局部变量表，不经过栈）
    /// 格式: iinc <index> <const>
    /// 示例: i++; 或 i += 5; 编译为 iinc 1 1 或 iinc 1 5
    /// 这是唯一直接修改局部变量表而不通过栈的指令
    pub const IINC: u8 = 0x84;

    // ============ 类型转换指令 (Type Conversion) ============
    // 用于不同基本类型之间的转换
    // 命名规则：<源类型>2<目标类型>，如i2l表示int转long

    /// 0x85 - int转long（扩展转换，无损）
    pub const I2L: u8 = 0x85;
    /// 0x86 - int转float（可能损失精度）
    pub const I2F: u8 = 0x86;
    /// 0x87 - int转double
    pub const I2D: u8 = 0x87;

    /// 0x88 - long转int（窄化转换，可能溢出）
    pub const L2I: u8 = 0x88;
    pub const L2F: u8 = 0x89;
    pub const L2D: u8 = 0x8a;

    pub const F2I: u8 = 0x8b;
    pub const F2L: u8 = 0x8c;
    pub const F2D: u8 = 0x8d;

    pub const D2I: u8 = 0x8e;
    pub const D2L: u8 = 0x8f;
    pub const D2F: u8 = 0x90;

    /// 0x91 - int转byte（保留低8位）
    pub const I2B: u8 = 0x91;
    /// 0x92 - int转char（保留低16位，无符号）
    pub const I2C: u8 = 0x92;
    /// 0x93 - int转short（保留低16位，有符号）
    pub const I2S: u8 = 0x93;

    // ============ 比较指令 (Comparison) ============
    // 比较两个值，将结果压入栈
    // long/float/double需要专门的比较指令，int直接用条件跳转指令

    /// 0x94 - long比较
    /// 栈变化: ..., value1, value2 → ..., result
    /// result: value1 > value2 时为1, value1 == value2 时为0, value1 < value2 时为-1
    pub const LCMP: u8 = 0x94;

    /// 0x95 - float比较（遇到NaN返回-1）
    pub const FCMPL: u8 = 0x95;
    /// 0x96 - float比较（遇到NaN返回1）
    pub const FCMPG: u8 = 0x96;

    /// 0x97 - double比较（遇到NaN返回-1）
    pub const DCMPL: u8 = 0x97;
    /// 0x98 - double比较（遇到NaN返回1）
    pub const DCMPG: u8 = 0x98;

    // ============ 条件跳转指令 (Conditional Branch) ============
    // 这是控制流的核心！用于实现if、while、for等控制结构
    // 所有跳转指令的操作数是相对于当前PC的偏移量（2字节有符号数）

    /// 0x99 - 如果栈顶int值等于0，跳转
    /// 格式: ifeq <branchoffset>
    /// 示例: if (x == 0) {...} 编译为 iload_1, ifeq label
    pub const IFEQ: u8 = 0x99;

    /// 0x9a - 如果栈顶int值不等于0，跳转
    /// 示例: if (x != 0) {...}
    pub const IFNE: u8 = 0x9a;

    /// 0x9b - 如果栈顶int值小于0，跳转
    /// 示例: if (x < 0) {...}
    pub const IFLT: u8 = 0x9b;

    /// 0x9c - 如果栈顶int值大于等于0，跳转
    /// 示例: if (x >= 0) {...}
    pub const IFGE: u8 = 0x9c;

    /// 0x9d - 如果栈顶int值大于0，跳转
    pub const IFGT: u8 = 0x9d;

    /// 0x9e - 如果栈顶int值小于等于0，跳转
    pub const IFLE: u8 = 0x9e;

    /// 0x9f - 比较栈顶两个int值，相等则跳转
    /// 栈变化: ..., value1, value2 → ...
    /// 示例: if (a == b) {...} 编译为 iload_1, iload_2, if_icmpeq label
    pub const IF_ICMPEQ: u8 = 0x9f;

    /// 0xa0 - 比较栈顶两个int值，不相等则跳转
    pub const IF_ICMPNE: u8 = 0xa0;

    /// 0xa1 - 比较栈顶两个int值，value1 < value2 则跳转
    pub const IF_ICMPLT: u8 = 0xa1;

    /// 0xa2 - 比较栈顶两个int值，value1 >= value2 则跳转
    pub const IF_ICMPGE: u8 = 0xa2;

    /// 0xa3 - 比较栈顶两个int值，value1 > value2 则跳转
    pub const IF_ICMPGT: u8 = 0xa3;

    /// 0xa4 - 比较栈顶两个int值，value1 <= value2 则跳转
    pub const IF_ICMPLE: u8 = 0xa4;

    /// 0xa5 - 比较栈顶两个引用，相等则跳转（比较的是引用地址）
    /// 示例: if (obj1 == obj2) {...}
    pub const IF_ACMPEQ: u8 = 0xa5;

    /// 0xa6 - 比较栈顶两个引用，不相等则跳转
    pub const IF_ACMPNE: u8 = 0xa6;

    // ============ 无条件跳转 (Unconditional Branch) ============

    /// 0xa7 - 无条件跳转到指定位置
    /// 格式: goto <branchoffset>
    /// 用于实现循环、break、continue等
    pub const GOTO: u8 = 0xa7;

    /// 0xa8 - 跳转到子例程（已废弃，不推荐使用）
    pub const JSR: u8 = 0xa8;

    /// 0xa9 - 从子例程返回（已废弃）
    pub const RET: u8 = 0xa9;

    // ============ 表跳转 (Table Switch) ============
    // 用于实现switch语句

    /// 0xaa - 表跳转（case值连续时使用）
    /// 格式复杂，包含default、low、high和跳转表
    /// 示例: switch(x) { case 0: ... case 1: ... case 2: ... }
    pub const TABLESWITCH: u8 = 0xaa;

    /// 0xab - 查找跳转（case值稀疏时使用）
    /// 使用键值对数组查找匹配的case
    pub const LOOKUPSWITCH: u8 = 0xab;

    // ============ 返回指令 (Return) ============
    // 从方法返回到调用者
    // 返回指令会结束当前方法，将返回值（如果有）传递给调用者

    /// 0xac - 返回int值
    /// 栈变化: ..., value → [empty]
    /// 示例: return x; 在int方法中编译为 iload_1, ireturn
    pub const IRETURN: u8 = 0xac;

    /// 0xad - 返回long值
    pub const LRETURN: u8 = 0xad;

    /// 0xae - 返回float值
    pub const FRETURN: u8 = 0xae;

    /// 0xaf - 返回double值
    pub const DRETURN: u8 = 0xaf;

    /// 0xb0 - 返回引用值
    pub const ARETURN: u8 = 0xb0;

    /// 0xb1 - 从void方法返回
    /// 示例: void方法结尾或显式return;
    pub const RETURN: u8 = 0xb1;

    // ============ 字段访问指令 (Field Access) ============
    // 用于读写对象字段和静态字段

    /// 0xb2 - 获取类的静态字段值
    /// 格式: getstatic <indexbyte1> <indexbyte2>
    /// 示例: int x = MyClass.staticField; 编译为 getstatic MyClass.staticField
    pub const GETSTATIC: u8 = 0xb2;

    /// 0xb3 - 设置类的静态字段值
    /// 示例: MyClass.staticField = 10;
    pub const PUTSTATIC: u8 = 0xb3;

    /// 0xb4 - 获取对象实例字段值
    /// 栈变化: ..., objectref → ..., value
    /// 示例: int x = obj.field; 编译为 aload_1, getfield obj.field
    pub const GETFIELD: u8 = 0xb4;

    /// 0xb5 - 设置对象实例字段值
    /// 栈变化: ..., objectref, value → ...
    pub const PUTFIELD: u8 = 0xb5;

    // ============ 方法调用指令 (Method Invocation) ============
    // 这是JVM中最复杂的部分！5种不同的方法调用方式

    /// 0xb6 - 调用实例方法（动态分派，根据对象实际类型调用）
    /// 格式: invokevirtual <indexbyte1> <indexbyte2>
    /// 示例: obj.method(); 编译为 aload_1, invokevirtual obj.method
    /// 这是最常见的方法调用，支持多态
    pub const INVOKEVIRTUAL: u8 = 0xb6;

    /// 0xb7 - 调用特殊方法（构造器、私有方法、父类方法）
    /// 不进行动态分派，直接调用指定方法
    /// 示例: super.method(); 或 new Object(); (调用<init>)
    pub const INVOKESPECIAL: u8 = 0xb7;

    /// 0xb8 - 调用静态方法
    /// 示例: Math.max(a, b); 编译为 iload_1, iload_2, invokestatic Math.max
    pub const INVOKESTATIC: u8 = 0xb8;

    /// 0xb9 - 调用接口方法（动态查找实现）
    /// 格式: invokeinterface <indexbyte1> <indexbyte2> <count> <0>
    pub const INVOKEINTERFACE: u8 = 0xb9;

    /// 0xba - 动态方法调用（Java 7引入，支持lambda等）
    pub const INVOKEDYNAMIC: u8 = 0xba;

    // ============ 对象和数组指令 (Object/Array) ============

    /// 0xbb - 创建新对象实例
    /// 格式: new <indexbyte1> <indexbyte2>
    /// 示例: new Object(); 编译为 new Object, dup, invokespecial Object.<init>
    /// 注意：new只分配内存，还需要调用<init>构造器
    pub const NEW: u8 = 0xbb;

    /// 0xbc - 创建基本类型数组
    /// 格式: newarray <atype>
    /// 示例: new int[10]; 编译为 bipush 10, newarray int
    pub const NEWARRAY: u8 = 0xbc;

    /// 0xbd - 创建引用类型数组
    /// 示例: new String[10];
    pub const ANEWARRAY: u8 = 0xbd;

    /// 0xbe - 获取数组长度
    /// 栈变化: ..., arrayref → ..., length
    /// 示例: int len = arr.length;
    pub const ARRAYLENGTH: u8 = 0xbe;

    /// 0xbf - 抛出异常
    /// 栈变化: ..., objectref → objectref
    /// 示例: throw new Exception();
    pub const ATHROW: u8 = 0xbf;

    /// 0xc0 - 类型检查并转换
    /// 示例: (String) obj;
    pub const CHECKCAST: u8 = 0xc0;

    /// 0xc1 - 判断对象是否是某个类的实例
    /// 示例: obj instanceof String;
    pub const INSTANCEOF: u8 = 0xc1;

    // ============ 同步指令 (Synchronization) ============
    // 用于实现synchronized关键字

    /// 0xc2 - 进入监视器（获取锁）
    /// 示例: synchronized(obj) { ... }
    pub const MONITORENTER: u8 = 0xc2;

    /// 0xc3 - 退出监视器（释放锁）
    pub const MONITOREXIT: u8 = 0xc3;

    // ============ 扩展指令 (Extended) ============

    /// 0xc4 - 扩展局部变量索引（将下一条指令的索引扩展到16位）
    pub const WIDE: u8 = 0xc4;

    /// 0xc5 - 创建多维数组
    /// 示例: new int[3][4][5];
    pub const MULTIANEWARRAY: u8 = 0xc5;

    /// 0xc6 - 如果引用为null则跳转
    /// 示例: if (obj == null) {...}
    pub const IFNULL: u8 = 0xc6;

    /// 0xc7 - 如果引用不为null则跳转
    pub const IFNONNULL: u8 = 0xc7;

    /// 0xc8 - 无条件跳转（宽索引版本，4字节偏移）
    pub const GOTO_W: u8 = 0xc8;

    /// 0xc9 - 跳转到子例程（宽索引版本，已废弃）
    pub const JSR_W: u8 = 0xc9;
}

/// 获取指令名称（用于调试和日志输出）
/// 将字节码操作码转换为人类可读的指令名称
pub fn get_instruction_name(opcode: u8) -> &'static str {
    use opcodes::*;
    match opcode {
        // 常量指令
        NOP => "nop",
        ACONST_NULL => "aconst_null",
        ICONST_M1 => "iconst_m1",
        ICONST_0 => "iconst_0",
        ICONST_1 => "iconst_1",
        ICONST_2 => "iconst_2",
        ICONST_3 => "iconst_3",
        ICONST_4 => "iconst_4",
        ICONST_5 => "iconst_5",
        LCONST_0 => "lconst_0",
        LCONST_1 => "lconst_1",
        FCONST_0 => "fconst_0",
        FCONST_1 => "fconst_1",
        FCONST_2 => "fconst_2",
        DCONST_0 => "dconst_0",
        DCONST_1 => "dconst_1",
        BIPUSH => "bipush",
        SIPUSH => "sipush",
        LDC => "ldc",
        LDC_W => "ldc_w",
        LDC2_W => "ldc2_w",

        // 加载指令
        ILOAD => "iload",
        LLOAD => "lload",
        FLOAD => "fload",
        DLOAD => "dload",
        ALOAD => "aload",
        ILOAD_0 => "iload_0",
        ILOAD_1 => "iload_1",
        ILOAD_2 => "iload_2",
        ILOAD_3 => "iload_3",
        LLOAD_0 => "lload_0",
        LLOAD_1 => "lload_1",
        LLOAD_2 => "lload_2",
        LLOAD_3 => "lload_3",
        FLOAD_0 => "fload_0",
        FLOAD_1 => "fload_1",
        FLOAD_2 => "fload_2",
        FLOAD_3 => "fload_3",
        DLOAD_0 => "dload_0",
        DLOAD_1 => "dload_1",
        DLOAD_2 => "dload_2",
        DLOAD_3 => "dload_3",
        ALOAD_0 => "aload_0",
        ALOAD_1 => "aload_1",
        ALOAD_2 => "aload_2",
        ALOAD_3 => "aload_3",

        // 数组加载
        IALOAD => "iaload",
        LALOAD => "laload",
        FALOAD => "faload",
        DALOAD => "daload",
        AALOAD => "aaload",
        BALOAD => "baload",
        CALOAD => "caload",
        SALOAD => "saload",

        // 存储指令
        ISTORE => "istore",
        LSTORE => "lstore",
        FSTORE => "fstore",
        DSTORE => "dstore",
        ASTORE => "astore",
        ISTORE_0 => "istore_0",
        ISTORE_1 => "istore_1",
        ISTORE_2 => "istore_2",
        ISTORE_3 => "istore_3",
        LSTORE_0 => "lstore_0",
        LSTORE_1 => "lstore_1",
        LSTORE_2 => "lstore_2",
        LSTORE_3 => "lstore_3",
        FSTORE_0 => "fstore_0",
        FSTORE_1 => "fstore_1",
        FSTORE_2 => "fstore_2",
        FSTORE_3 => "fstore_3",
        DSTORE_0 => "dstore_0",
        DSTORE_1 => "dstore_1",
        DSTORE_2 => "dstore_2",
        DSTORE_3 => "dstore_3",
        ASTORE_0 => "astore_0",
        ASTORE_1 => "astore_1",
        ASTORE_2 => "astore_2",
        ASTORE_3 => "astore_3",

        // 栈操作
        POP => "pop",
        POP2 => "pop2",
        DUP => "dup",
        DUP_X1 => "dup_x1",
        DUP_X2 => "dup_x2",
        DUP2 => "dup2",
        DUP2_X1 => "dup2_x1",
        DUP2_X2 => "dup2_x2",
        SWAP => "swap",

        // 算术指令
        IADD => "iadd",
        LADD => "ladd",
        FADD => "fadd",
        DADD => "dadd",
        ISUB => "isub",
        LSUB => "lsub",
        FSUB => "fsub",
        DSUB => "dsub",
        IMUL => "imul",
        LMUL => "lmul",
        FMUL => "fmul",
        DMUL => "dmul",
        IDIV => "idiv",
        LDIV => "ldiv",
        FDIV => "fdiv",
        DDIV => "ddiv",
        IREM => "irem",
        LREM => "lrem",
        FREM => "frem",
        DREM => "drem",
        INEG => "ineg",
        LNEG => "lneg",
        FNEG => "fneg",
        DNEG => "dneg",

        // 位运算
        ISHL => "ishl",
        LSHL => "lshl",
        ISHR => "ishr",
        LSHR => "lshr",
        IUSHR => "iushr",
        LUSHR => "lushr",
        IAND => "iand",
        LAND => "land",
        IOR => "ior",
        LOR => "lor",
        IXOR => "ixor",
        LXOR => "lxor",
        IINC => "iinc",

        // 类型转换
        I2L => "i2l",
        I2F => "i2f",
        I2D => "i2d",
        L2I => "l2i",
        L2F => "l2f",
        L2D => "l2d",
        F2I => "f2i",
        F2L => "f2l",
        F2D => "f2d",
        D2I => "d2i",
        D2L => "d2l",
        D2F => "d2f",
        I2B => "i2b",
        I2C => "i2c",
        I2S => "i2s",

        // 比较
        LCMP => "lcmp",
        FCMPL => "fcmpl",
        FCMPG => "fcmpg",
        DCMPL => "dcmpl",
        DCMPG => "dcmpg",

        // 条件跳转
        IFEQ => "ifeq",
        IFNE => "ifne",
        IFLT => "iflt",
        IFGE => "ifge",
        IFGT => "ifgt",
        IFLE => "ifle",
        IF_ICMPEQ => "if_icmpeq",
        IF_ICMPNE => "if_icmpne",
        IF_ICMPLT => "if_icmplt",
        IF_ICMPGE => "if_icmpge",
        IF_ICMPGT => "if_icmpgt",
        IF_ICMPLE => "if_icmple",
        IF_ACMPEQ => "if_acmpeq",
        IF_ACMPNE => "if_acmpne",

        // 跳转
        GOTO => "goto",
        JSR => "jsr",
        RET => "ret",
        TABLESWITCH => "tableswitch",
        LOOKUPSWITCH => "lookupswitch",

        // 返回
        IRETURN => "ireturn",
        LRETURN => "lreturn",
        FRETURN => "freturn",
        DRETURN => "dreturn",
        ARETURN => "areturn",
        RETURN => "return",

        // 字段访问
        GETSTATIC => "getstatic",
        PUTSTATIC => "putstatic",
        GETFIELD => "getfield",
        PUTFIELD => "putfield",

        // 方法调用
        INVOKEVIRTUAL => "invokevirtual",
        INVOKESPECIAL => "invokespecial",
        INVOKESTATIC => "invokestatic",
        INVOKEINTERFACE => "invokeinterface",
        INVOKEDYNAMIC => "invokedynamic",

        // 对象和数组
        NEW => "new",
        NEWARRAY => "newarray",
        ANEWARRAY => "anewarray",
        ARRAYLENGTH => "arraylength",
        ATHROW => "athrow",
        CHECKCAST => "checkcast",
        INSTANCEOF => "instanceof",

        // 同步
        MONITORENTER => "monitorenter",
        MONITOREXIT => "monitorexit",

        // 扩展
        WIDE => "wide",
        MULTIANEWARRAY => "multianewarray",
        IFNULL => "ifnull",
        IFNONNULL => "ifnonnull",
        GOTO_W => "goto_w",
        JSR_W => "jsr_w",

        _ => "unknown",
    }
}
