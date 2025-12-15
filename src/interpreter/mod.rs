//! # 字节码解释器
//!
//! 解释器负责执行Java字节码指令。
//!
//! ## 学习要点
//! - JVM是基于栈的虚拟机
//! - 每条指令都对操作数栈进行操作
//! - 需要理解不同指令的语义
//!
//! ## 主要指令分类
//! - 常量指令：将常量压入栈（iconst, ldc等）
//! - 加载指令：从局部变量表加载到栈（iload, aload等）
//! - 存储指令：从栈存储到局部变量表（istore, astore等）
//! - 运算指令：算术和逻辑运算（iadd, isub等）
//! - 类型转换：基本类型转换（i2l, f2i等）
//! - 对象操作：创建对象、访问字段（new, getfield等）
//! - 方法调用：调用方法（invokevirtual, invokestatic等）
//! - 控制转移：分支和跳转（if_icmpeq, goto等）
//! - 返回指令：方法返回（ireturn, return等）

pub mod instructions;

use crate::classfile::ClassFile;
use crate::runtime::frame::JvmValue;
use crate::runtime::{Frame, Heap, JvmThread, Metaspace};
use crate::Result;
use anyhow::anyhow;

/// 指令执行控制
enum InstructionControl {
    /// 继续执行下一条指令
    Continue,
    /// 方法返回，携带返回值（如果有）
    Return(Option<JvmValue>),
}

/// 解释器
pub struct Interpreter {
    /// 堆
    pub heap: Heap,
    /// 当前线程
    pub thread: JvmThread,
    /// 方法区 - 存储所有类的元数据
    pub metaspace: Metaspace,
}

impl Interpreter {
    /// 创建新的解释器
    pub fn new() -> Self {
        Interpreter {
            heap: Heap::new(),
            thread: JvmThread::new(),
            metaspace: Metaspace::new(),
        }
    }

    /// 执行方法（带类名上下文）- 新版显式栈实现
    /// 返回方法的返回值（如果有）
    pub fn execute_method_with_class(
        &mut self,
        class_name: &str,
        code: &[u8],
        max_locals: usize,
        max_stack: usize,
    ) -> Result<Option<JvmValue>> {
        // 创建初始栈帧
        let frame = Frame::new_with_context(
            max_locals,
            max_stack,
            class_name.to_string(),
            code.to_vec(),
            None, // 顶层方法没有返回地址
        );

        // 压入栈帧到线程
        self.thread.push_frame(frame);
        self.thread.pc = 0;

        // 主执行循环：运行直到栈为空
        let mut return_value = None;
        while self.thread.stack_depth() > 0 {
            // 获取当前字节码
            let code = self.thread.current_code()?.to_vec();
            let pc = self.thread.pc;

            if pc >= code.len() {
                return Err(anyhow!("PC out of bounds: {} >= {}", pc, code.len()));
            }

            let opcode = code[pc];
            let control = self.execute_instruction_explicit(opcode)?;

            match control {
                InstructionControl::Continue => {}
                InstructionControl::Return(val) => {
                    // 方法返回
                    return_value = val;
                    break;
                }
            }
        }

        Ok(return_value)
    }

    /// 执行单条指令 - 显式栈版本（使用线程级PC）
    fn execute_instruction_explicit(&mut self, opcode: u8) -> Result<InstructionControl> {
        use instructions::opcodes::*;

        // 克隆需要的数据以避免借用冲突
        let code = self.thread.current_code()?.to_vec();
        let pc = self.thread.pc;
        let class_name = self.thread.current_frame()?.class_name.clone();

        match opcode {
            NOP => {
                self.thread.pc += 1;
            }
            NEW => {
                let class_index = u16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                // 使用 ClassMetadata 的 resolve_class_ref
                let target_class_name = {
                    let class_meta: &mut crate::runtime::ClassMetadata =
                        self.metaspace.get_class_mut(&class_name)?;
                    class_meta.resolve_class_ref(class_index)?
                };
                let ptr = self.heap.allocate(target_class_name);
                self.thread
                    .current_frame_mut()?
                    .push(JvmValue::Reference(Some(ptr)));
                self.thread.pc += 3;
            }
            PUTFIELD => {
                let field_index = u16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                let class_meta: &mut crate::runtime::ClassMetadata =
                    self.metaspace.get_class_mut(&class_name)?;
                let field_ref = class_meta.resolve_field_ref(field_index)?;
                let value = self.thread.current_frame_mut()?.pop()?;
                let obj_ref = self
                    .thread
                    .current_frame_mut()?
                    .pop_ref()?
                    .ok_or(anyhow!("invalid ref"))?;
                self.heap
                    .set_field(obj_ref, field_ref.field_name.clone(), value)?;
                self.thread.pc += 3;
            }
            GETFIELD => {
                let field_index: u16 = u16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                let class_meta: &mut crate::runtime::ClassMetadata =
                    self.metaspace.get_class_mut(&class_name)?;
                let field_ref = class_meta.resolve_field_ref(field_index)?;
                let obj_ref = self
                    .thread
                    .current_frame_mut()?
                    .pop_ref()?
                    .ok_or(anyhow!("invalid ref"))?;
                let val = self.heap.get_field(obj_ref, &field_ref.field_name)?;
                self.thread.current_frame_mut()?.push(val.clone());
                self.thread.pc += 3;
            }

            INVOKESPECIAL => {
                let method_index: u16 = u16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                let class_meta: &mut crate::runtime::ClassMetadata =
                    self.metaspace.get_class_mut(&class_name)?;
                let method_ref = class_meta.resolve_method_ref(method_index)?;
                // 2. 检查目标类是否已加载
                // 作弊版：跳过 java.* 系统类检查
                let is_system_class = method_ref.class_name.starts_with("java/");
                if !is_system_class && !self.metaspace.is_class_loaded(&method_ref.class_name) {
                    return Err(anyhow!(
                        "Class {} not loaded. Please load it first using interpreter.load_class()",
                        method_ref.class_name
                    ));
                }

                // 3. 查找目标方法（如果是系统类，跳过）
                if is_system_class {
                    // 系统类方法调用：假装调用成功，什么都不做
                    // 这适用于 super() 调用 Object.<init>
                    self.thread.pc += 3;
                    return Ok(InstructionControl::Continue);
                }

                // 4. 查找目标方法（用户类）
                let target_class = self.metaspace.get_class(&method_ref.class_name)?;
                let method_key = format!("{}:{}", method_ref.method_name, method_ref.descriptor);
                let method = target_class
                    .methods
                    .get(&method_key)
                    .ok_or_else(|| {
                        anyhow!("Method not found: {}.{}", method_ref.class_name, method_key)
                    })?
                    .clone();
                // 4. 从操作数栈弹出参数
                let arg_count = Self::parse_arg_count(&method.descriptor);
                let mut args: Vec<JvmValue> = Vec::new();
                for _ in 0..arg_count {
                    args.push(self.thread.current_frame_mut()?.pop()?);
                }
                args.reverse(); // 栈是LIFO，需要反转
                                // 5. ⭐ 关键区别：弹出 objectref (this 引用)
                let objectref = self.thread.current_frame_mut()?.pop()?;

                // 6. 创建新栈帧并设置参数
                let mut new_frame = Frame::new_with_context(
                    method.max_locals,
                    method.max_stack,
                    method_ref.class_name.clone(),
                    method.code.clone(),
                    Some(pc + 3), // 返回地址
                );

                // 7. ⭐ 关键区别：设置 this (local[0])
                new_frame.set_local(0, objectref)?;
                // 8. 设置参数（从 local[1] 开始）
                for (i, arg) in args.into_iter().enumerate() {
                    new_frame.set_local(i + 1, arg)?; // ← 注意：i+1，因为 local[0] 是 this
                }
                // 9. 压入新栈帧到线程栈
                self.thread.push_frame(new_frame);
                // 10. 设置PC为0，开始执行被调用方法
                self.thread.pc = 0;
            }
            DUP => {
                let value = self.thread.current_frame_mut()?.pop()?;
                self.thread.current_frame_mut()?.push(value.clone());
                self.thread.current_frame_mut()?.push(value);
                self.thread.pc += 1;
            }

            // ==================== 常量指令 ====================
            ICONST_M1 => {
                self.thread.current_frame_mut()?.push(JvmValue::Int(-1));
                self.thread.pc += 1;
            }
            ICONST_0 => {
                self.thread.current_frame_mut()?.push(JvmValue::Int(0));
                self.thread.pc += 1;
            }
            ICONST_1 => {
                self.thread.current_frame_mut()?.push(JvmValue::Int(1));
                self.thread.pc += 1;
            }
            ICONST_2 => {
                self.thread.current_frame_mut()?.push(JvmValue::Int(2));
                self.thread.pc += 1;
            }
            ICONST_3 => {
                self.thread.current_frame_mut()?.push(JvmValue::Int(3));
                self.thread.pc += 1;
            }
            ICONST_4 => {
                self.thread.current_frame_mut()?.push(JvmValue::Int(4));
                self.thread.pc += 1;
            }
            ICONST_5 => {
                self.thread.current_frame_mut()?.push(JvmValue::Int(5));
                self.thread.pc += 1;
            }

            BIPUSH => {
                let value = code[pc + 1] as i8;
                self.thread
                    .current_frame_mut()?
                    .push(JvmValue::Int(value as i32));
                self.thread.pc += 2;
            }

            SIPUSH => {
                let value = i16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                self.thread
                    .current_frame_mut()?
                    .push(JvmValue::Int(value as i32));
                self.thread.pc += 3;
            }
            ALOAD | ILOAD => {
                let index = code[pc + 1] as usize;
                let value = self.thread.current_frame()?.get_local(index)?.clone();
                self.thread.current_frame_mut()?.push(value);
                self.thread.pc += 2;
            }

            ALOAD_0 | ALOAD_1 | ALOAD_2 | ALOAD_3 => {
                let index = (opcode - ALOAD_0) as usize;
                let value = self.thread.current_frame()?.get_local(index)?.clone();
                self.thread.current_frame_mut()?.push(value);
                self.thread.pc += 1;
            }
            // ==================== 加载指令 ====================
            ILOAD_0 | ILOAD_1 | ILOAD_2 | ILOAD_3 => {
                let index = (opcode - ILOAD_0) as usize;
                let value = self.thread.current_frame()?.get_local(index)?.clone();
                self.thread.current_frame_mut()?.push(value);
                self.thread.pc += 1;
            }

            ASTORE_0 | ASTORE_1 | ASTORE_2 | ASTORE_3 => {
                let index = (opcode - ASTORE_0) as usize;
                let value = self.thread.current_frame_mut()?.pop()?;
                self.thread.current_frame_mut()?.set_local(index, value)?;
                self.thread.pc += 1;
            }
            // ==================== 存储指令 ====================
            ISTORE_0 | ISTORE_1 | ISTORE_2 | ISTORE_3 => {
                let index = (opcode - ISTORE_0) as usize;
                let value = self.thread.current_frame_mut()?.pop()?;
                self.thread.current_frame_mut()?.set_local(index, value)?;
                self.thread.pc += 1;
            }

            // ==================== 运算指令 ====================
            IADD => {
                let v2 = self.thread.current_frame_mut()?.pop_int()?;
                let v1 = self.thread.current_frame_mut()?.pop_int()?;
                self.thread
                    .current_frame_mut()?
                    .push(JvmValue::Int(v1 + v2));
                self.thread.pc += 1;
            }

            ISUB => {
                let v2 = self.thread.current_frame_mut()?.pop_int()?;
                let v1 = self.thread.current_frame_mut()?.pop_int()?;
                self.thread
                    .current_frame_mut()?
                    .push(JvmValue::Int(v1 - v2));
                self.thread.pc += 1;
            }

            IMUL => {
                let v2 = self.thread.current_frame_mut()?.pop_int()?;
                let v1 = self.thread.current_frame_mut()?.pop_int()?;
                self.thread
                    .current_frame_mut()?
                    .push(JvmValue::Int(v1 * v2));
                self.thread.pc += 1;
            }

            IDIV => {
                let v2 = self.thread.current_frame_mut()?.pop_int()?;
                let v1 = self.thread.current_frame_mut()?.pop_int()?;
                if v2 == 0 {
                    return Err(anyhow!("Division by zero"));
                }
                self.thread
                    .current_frame_mut()?
                    .push(JvmValue::Int(v1 / v2));
                self.thread.pc += 1;
            }

            // ==================== 控制流指令 ====================
            IFEQ => {
                let offset = i16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                let value = self.thread.current_frame_mut()?.pop_int()?;
                if value == 0 {
                    self.thread.pc = (pc as i32 + offset as i32) as usize;
                } else {
                    self.thread.pc += 3;
                }
            }

            IFNE => {
                let offset = i16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                let value = self.thread.current_frame_mut()?.pop_int()?;
                if value != 0 {
                    self.thread.pc = (pc as i32 + offset as i32) as usize;
                } else {
                    self.thread.pc += 3;
                }
            }

            IFLT => {
                let offset = i16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                let value = self.thread.current_frame_mut()?.pop_int()?;
                if value < 0 {
                    self.thread.pc = (pc as i32 + offset as i32) as usize;
                } else {
                    self.thread.pc += 3;
                }
            }

            IFGE => {
                let offset = i16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                let value = self.thread.current_frame_mut()?.pop_int()?;
                if value >= 0 {
                    self.thread.pc = (pc as i32 + offset as i32) as usize;
                } else {
                    self.thread.pc += 3;
                }
            }

            IFGT => {
                let offset = i16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                let value = self.thread.current_frame_mut()?.pop_int()?;
                if value > 0 {
                    self.thread.pc = (pc as i32 + offset as i32) as usize;
                } else {
                    self.thread.pc += 3;
                }
            }

            IFLE => {
                let offset = i16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                let value = self.thread.current_frame_mut()?.pop_int()?;
                if value <= 0 {
                    self.thread.pc = (pc as i32 + offset as i32) as usize;
                } else {
                    self.thread.pc += 3;
                }
            }

            IF_ICMPEQ => {
                let offset = i16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                let v2 = self.thread.current_frame_mut()?.pop_int()?;
                let v1 = self.thread.current_frame_mut()?.pop_int()?;
                if v1 == v2 {
                    self.thread.pc = (pc as i32 + offset as i32) as usize;
                } else {
                    self.thread.pc += 3;
                }
            }

            IF_ICMPNE => {
                let offset = i16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                let v2 = self.thread.current_frame_mut()?.pop_int()?;
                let v1 = self.thread.current_frame_mut()?.pop_int()?;
                if v1 != v2 {
                    self.thread.pc = (pc as i32 + offset as i32) as usize;
                } else {
                    self.thread.pc += 3;
                }
            }

            IF_ICMPLT => {
                let offset = i16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                let v2 = self.thread.current_frame_mut()?.pop_int()?;
                let v1 = self.thread.current_frame_mut()?.pop_int()?;
                if v1 < v2 {
                    self.thread.pc = (pc as i32 + offset as i32) as usize;
                } else {
                    self.thread.pc += 3;
                }
            }

            IF_ICMPGE => {
                let offset = i16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                let v2 = self.thread.current_frame_mut()?.pop_int()?;
                let v1 = self.thread.current_frame_mut()?.pop_int()?;
                if v1 >= v2 {
                    self.thread.pc = (pc as i32 + offset as i32) as usize;
                } else {
                    self.thread.pc += 3;
                }
            }

            IF_ICMPGT => {
                let offset = i16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                let v2 = self.thread.current_frame_mut()?.pop_int()?;
                let v1 = self.thread.current_frame_mut()?.pop_int()?;
                if v1 > v2 {
                    self.thread.pc = (pc as i32 + offset as i32) as usize;
                } else {
                    self.thread.pc += 3;
                }
            }

            IF_ICMPLE => {
                let offset = i16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                let v2 = self.thread.current_frame_mut()?.pop_int()?;
                let v1 = self.thread.current_frame_mut()?.pop_int()?;
                if v1 <= v2 {
                    self.thread.pc = (pc as i32 + offset as i32) as usize;
                } else {
                    self.thread.pc += 3;
                }
            }

            GOTO => {
                let offset = i16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                self.thread.pc = (pc as i32 + offset as i32) as usize;
            }

            // ==================== 方法调用指令 ====================
            INVOKESTATIC => {
                let index = u16::from_be_bytes([code[pc + 1], code[pc + 2]]);

                // 1. 解析方法引用
                let method_ref = {
                    let class_meta = self.metaspace.get_class_mut(&class_name)?;
                    class_meta.resolve_method_ref(index)?
                };

                // 2. 检查类是否已加载
                // 作弊版：跳过 java.* 系统类检查
                let is_system_class = method_ref.class_name.starts_with("java/");
                if !is_system_class && !self.metaspace.is_class_loaded(&method_ref.class_name) {
                    return Err(anyhow!(
                        "Class {} not loaded. Please load it first using interpreter.load_class()",
                        method_ref.class_name
                    ));
                }

                // 3. 查找目标方法（如果是系统类，跳过）
                if is_system_class {
                    // 系统类静态方法调用：假装调用成功，什么都不做
                    self.thread.pc += 3;
                    return Ok(InstructionControl::Continue);
                }

                // 4. 查找目标方法（用户类）
                let target_class = self.metaspace.get_class(&method_ref.class_name)?;
                let method_key = format!("{}:{}", method_ref.method_name, method_ref.descriptor);
                let method = target_class
                    .methods
                    .get(&method_key)
                    .ok_or_else(|| {
                        anyhow!("Method not found: {}.{}", method_ref.class_name, method_key)
                    })?
                    .clone();

                // 4. 从操作数栈弹出参数
                let arg_count = Self::parse_arg_count(&method.descriptor);
                let mut args: Vec<JvmValue> = Vec::new();
                for _ in 0..arg_count {
                    args.push(self.thread.current_frame_mut()?.pop()?);
                }
                args.reverse(); // 栈是LIFO，需要反转

                // 5. 创建新栈帧并设置参数和返回地址
                let mut new_frame = Frame::new_with_context(
                    method.max_locals,
                    method.max_stack,
                    method_ref.class_name.clone(),
                    method.code.clone(),
                    Some(pc + 3), // 返回地址：invokestatic 后的下一条指令
                );

                for (i, arg) in args.into_iter().enumerate() {
                    new_frame.set_local(i, arg)?;
                }

                // 6. 压入新栈帧到线程栈
                self.thread.push_frame(new_frame);

                // 7. 设置PC为0，开始执行被调用方法
                self.thread.pc = 0;
            }

            // ==================== 字段访问指令 (作弊版调试支持) ====================
            GETSTATIC => {
                // 作弊版：专门处理 System.out
                // 格式: getstatic #index
                let _index = u16::from_be_bytes([code[pc + 1], code[pc + 2]]);

                // 简化实现：我们不真正解析常量池，直接假设这是 System.out
                // 压入一个特殊的引用值作为 PrintStream 对象
                self.thread
                    .current_frame_mut()?
                    .push(JvmValue::Reference(Some(0xFFFF))); // 特殊标记值

                self.thread.pc += 3;
            }

            INVOKEVIRTUAL => {
                // 作弊版：专门处理 println
                // 格式: invokevirtual #index
                let index = u16::from_be_bytes([code[pc + 1], code[pc + 2]]);

                // 解析方法引用，检查是否是 println
                let method_ref = {
                    let class_meta = self.metaspace.get_class_mut(&class_name)?;
                    class_meta.resolve_method_ref(index)?
                };

                if method_ref.method_name == "println" {
                    // 这是 println 调用！
                    // 参数顺序：objectref, [args...]

                    // 弹出参数（根据描述符判断）
                    let arg_count = Self::parse_arg_count(&method_ref.descriptor);
                    let mut args = Vec::new();
                    for _ in 0..arg_count {
                        args.push(self.thread.current_frame_mut()?.pop()?);
                    }
                    args.reverse();

                    // 弹出 objectref (System.out)
                    let _objectref = self.thread.current_frame_mut()?.pop()?;

                    // 打印参数（作弊版：直接打印值）
                    if args.len() == 1 {
                        match &args[0] {
                            JvmValue::Int(val) => println!("{}", val),
                            JvmValue::Long(val) => println!("{}", val),
                            JvmValue::Float(val) => println!("{}", val),
                            JvmValue::Double(val) => println!("{}", val),
                            JvmValue::Reference(Some(addr)) => println!("Reference@{:x}", addr),
                            JvmValue::Reference(None) => println!("null"),
                        }
                    } else if args.is_empty() {
                        // println() 无参数，打印空行
                        println!();
                    }
                    self.thread.pc += 3;
                } else {
                    return Err(anyhow!(
                        "INVOKEVIRTUAL not implemented for method: {}.{}",
                        method_ref.class_name,
                        method_ref.method_name
                    ));
                }
            }

            // ==================== 返回指令 ====================
            IRETURN => {
                // 1. 弹出返回值
                let return_value = self.thread.current_frame_mut()?.pop()?;

                // 2. 弹出当前栈帧
                let old_frame = self.thread.pop_frame()?;

                // 3. 如果还有调用者栈帧，恢复PC并压入返回值
                if self.thread.stack_depth() > 0 {
                    // 恢复调用者的PC
                    if let Some(return_addr) = old_frame.return_address {
                        self.thread.pc = return_addr;
                    } else {
                        return Err(anyhow!("Missing return address in frame"));
                    }

                    // 将返回值压入调用者的操作数栈
                    self.thread.current_frame_mut()?.push(return_value);
                } else {
                    // 顶层方法返回，携带返回值
                    return Ok(InstructionControl::Return(Some(return_value)));
                }
            }

            RETURN => {
                // void返回
                let old_frame = self.thread.pop_frame()?;

                if self.thread.stack_depth() > 0 {
                    // 恢复调用者的PC
                    if let Some(return_addr) = old_frame.return_address {
                        self.thread.pc = return_addr;
                    } else {
                        return Err(anyhow!("Missing return address in frame"));
                    }
                } else {
                    // 顶层方法返回
                    return Ok(InstructionControl::Return(None));
                }
            }

            _ => {
                return Err(anyhow!("Unknown opcode: 0x{:02X} at pc {}", opcode, pc));
            }
        }

        Ok(InstructionControl::Continue)
    }

    /// 在给定栈帧中执行方法（向后兼容，旧测试用）
    #[deprecated(note = "use execute_method_with_class instead")]
    pub fn execute_method_in_frame(
        &mut self,
        code: &[u8],
        frame: &mut Frame,
        class_name: &str,
    ) -> Result<Option<JvmValue>> {
        // 创建临时 PC（旧架构）
        let mut pc = 0;
        let mut return_value = None;

        while pc < code.len() {
            let opcode = code[pc];
            let control =
                self.execute_instruction_legacy(opcode, code, frame, &mut pc, class_name)?;
            match control {
                InstructionControl::Continue => {}
                InstructionControl::Return(val) => {
                    return_value = val;
                    break;
                }
            }
        }

        Ok(return_value)
    }

    /// 加载类到 Metaspace（如果尚未加载）
    pub fn load_class(&mut self, class_file: ClassFile) -> Result<String> {
        let class_name = class_file.get_class_name()?;

        // 检查是否已加载
        if !self.metaspace.is_class_loaded(&class_name) {
            self.metaspace.load_class(class_file)?;
        }

        Ok(class_name)
    }

    /// 从常量池解析方法描述符中的参数个数
    /// 例如: "(II)I" -> 2, "(JD)V" -> 2 (long和double各占1个参数位)
    fn parse_arg_count(descriptor: &str) -> usize {
        let mut count = 0;
        let mut chars = descriptor.chars().skip(1); // 跳过开头的 '('

        while let Some(ch) = chars.next() {
            match ch {
                ')' => break, // 参数列表结束
                'B' | 'C' | 'S' | 'I' | 'F' | 'Z' => count += 1,
                'J' | 'D' => count += 1, // long 和 double
                'L' => {
                    // 引用类型，跳到分号
                    while let Some(c) = chars.next() {
                        if c == ';' {
                            break;
                        }
                    }
                    count += 1;
                }
                '[' => {
                    // 数组，需要读取后面的类型
                    if let Some(next) = chars.next() {
                        if next == 'L' {
                            while let Some(c) = chars.next() {
                                if c == ';' {
                                    break;
                                }
                            }
                        }
                    }
                    count += 1;
                }
                _ => {}
            }
        }

        count
    }

    /// 执行方法（向后兼容，旧测试用）
    #[deprecated(note = "use execute_method_with_class instead")]
    pub fn execute_method(
        &mut self,
        code: &[u8],
        max_locals: usize,
        max_stack: usize,
    ) -> Result<Option<JvmValue>> {
        let mut frame = Frame::new(max_locals, max_stack);
        let mut pc = 0;
        let mut return_value = None;

        while pc < code.len() {
            let opcode = code[pc];
            let control = self.execute_instruction_legacy(opcode, code, &mut frame, &mut pc, "")?;
            match control {
                InstructionControl::Continue => {}
                InstructionControl::Return(val) => {
                    return_value = val;
                    break;
                }
            }
        }

        Ok(return_value)
    }

    /// 执行单条指令（旧架构，向后兼容）
    #[deprecated(note = "use execute_instruction_explicit instead")]
    fn execute_instruction_legacy(
        &mut self,
        opcode: u8,
        code: &[u8],
        frame: &mut Frame,
        pc: &mut usize,
        current_class: &str,
    ) -> Result<InstructionControl> {
        use instructions::opcodes::*;

        match opcode {
            NOP => {
                // 无操作
                *pc += 1;
            }

            // ==================== 常量指令 ====================
            ICONST_M1 => {
                frame.push(crate::runtime::frame::JvmValue::Int(-1));
                *pc += 1;
            }
            ICONST_0 => {
                frame.push(crate::runtime::frame::JvmValue::Int(0));
                *pc += 1;
            }
            ICONST_1 => {
                frame.push(crate::runtime::frame::JvmValue::Int(1));
                *pc += 1;
            }
            ICONST_2 => {
                frame.push(crate::runtime::frame::JvmValue::Int(2));
                *pc += 1;
            }
            ICONST_3 => {
                frame.push(crate::runtime::frame::JvmValue::Int(3));
                *pc += 1;
            }
            ICONST_4 => {
                frame.push(crate::runtime::frame::JvmValue::Int(4));
                *pc += 1;
            }
            ICONST_5 => {
                frame.push(crate::runtime::frame::JvmValue::Int(5));
                *pc += 1;
            }

            BIPUSH => {
                let value = code[*pc + 1] as i8;
                frame.push(crate::runtime::frame::JvmValue::Int(value as i32));
                *pc += 2;
            }

            SIPUSH => {
                let value = i16::from_be_bytes([code[*pc + 1], code[*pc + 2]]);
                frame.push(crate::runtime::frame::JvmValue::Int(value as i32));
                *pc += 3;
            }

            // ==================== 加载指令 ====================
            ILOAD_0 | ILOAD_1 | ILOAD_2 | ILOAD_3 => {
                let index = (opcode - ILOAD_0) as usize;
                let value = frame.get_local(index)?.clone();
                frame.push(value);
                *pc += 1;
            }

            // ==================== 存储指令 ====================
            ISTORE_0 | ISTORE_1 | ISTORE_2 | ISTORE_3 => {
                let index = (opcode - ISTORE_0) as usize;
                let value = frame.pop()?;
                frame.set_local(index, value)?;
                *pc += 1;
            }

            // ==================== 运算指令 ====================
            IADD => {
                let v2 = frame.pop_int()?;
                let v1 = frame.pop_int()?;
                frame.push(crate::runtime::frame::JvmValue::Int(v1 + v2));
                *pc += 1;
            }

            ISUB => {
                let v2 = frame.pop_int()?;
                let v1 = frame.pop_int()?;
                frame.push(crate::runtime::frame::JvmValue::Int(v1 - v2));
                *pc += 1;
            }

            IMUL => {
                let v2 = frame.pop_int()?;
                let v1 = frame.pop_int()?;
                frame.push(crate::runtime::frame::JvmValue::Int(v1 * v2));
                *pc += 1;
            }

            IDIV => {
                let v2 = frame.pop_int()?;
                let v1 = frame.pop_int()?;
                if v2 == 0 {
                    return Err(anyhow!("Division by zero"));
                }
                frame.push(crate::runtime::frame::JvmValue::Int(v1 / v2));
                *pc += 1;
            }

            // ==================== 控制流指令 ====================

            // 零值比较跳转
            IFEQ => {
                let offset = i16::from_be_bytes([code[*pc + 1], code[*pc + 2]]);
                let value = frame.pop_int()?;
                if value == 0 {
                    *pc = (*pc as i32 + offset as i32) as usize;
                } else {
                    *pc += 3;
                }
            }

            IFNE => {
                let offset = i16::from_be_bytes([code[*pc + 1], code[*pc + 2]]);
                let value = frame.pop_int()?;
                if value != 0 {
                    *pc = (*pc as i32 + offset as i32) as usize;
                } else {
                    *pc += 3;
                }
            }

            IFLT => {
                let offset = i16::from_be_bytes([code[*pc + 1], code[*pc + 2]]);
                let value = frame.pop_int()?;
                if value < 0 {
                    *pc = (*pc as i32 + offset as i32) as usize;
                } else {
                    *pc += 3;
                }
            }

            IFGE => {
                let offset = i16::from_be_bytes([code[*pc + 1], code[*pc + 2]]);
                let value = frame.pop_int()?;
                if value >= 0 {
                    *pc = (*pc as i32 + offset as i32) as usize;
                } else {
                    *pc += 3;
                }
            }

            IFGT => {
                let offset = i16::from_be_bytes([code[*pc + 1], code[*pc + 2]]);
                let value = frame.pop_int()?;
                if value > 0 {
                    *pc = (*pc as i32 + offset as i32) as usize;
                } else {
                    *pc += 3;
                }
            }

            IFLE => {
                let offset = i16::from_be_bytes([code[*pc + 1], code[*pc + 2]]);
                let value = frame.pop_int()?;
                if value <= 0 {
                    *pc = (*pc as i32 + offset as i32) as usize;
                } else {
                    *pc += 3;
                }
            }

            // 两数比较跳转
            IF_ICMPEQ => {
                let offset = i16::from_be_bytes([code[*pc + 1], code[*pc + 2]]);
                let v2 = frame.pop_int()?;
                let v1 = frame.pop_int()?;
                if v1 == v2 {
                    *pc = (*pc as i32 + offset as i32) as usize;
                } else {
                    *pc += 3;
                }
            }

            IF_ICMPNE => {
                let offset = i16::from_be_bytes([code[*pc + 1], code[*pc + 2]]);
                let v2 = frame.pop_int()?;
                let v1 = frame.pop_int()?;
                if v1 != v2 {
                    *pc = (*pc as i32 + offset as i32) as usize;
                } else {
                    *pc += 3;
                }
            }

            IF_ICMPLT => {
                let offset = i16::from_be_bytes([code[*pc + 1], code[*pc + 2]]);
                let v2 = frame.pop_int()?;
                let v1 = frame.pop_int()?;
                if v1 < v2 {
                    *pc = (*pc as i32 + offset as i32) as usize;
                } else {
                    *pc += 3;
                }
            }

            IF_ICMPGE => {
                let offset = i16::from_be_bytes([code[*pc + 1], code[*pc + 2]]);
                let v2 = frame.pop_int()?;
                let v1 = frame.pop_int()?;
                if v1 >= v2 {
                    *pc = (*pc as i32 + offset as i32) as usize;
                } else {
                    *pc += 3;
                }
            }

            IF_ICMPGT => {
                let offset = i16::from_be_bytes([code[*pc + 1], code[*pc + 2]]);
                let v2 = frame.pop_int()?;
                let v1 = frame.pop_int()?;
                if v1 > v2 {
                    *pc = (*pc as i32 + offset as i32) as usize;
                } else {
                    *pc += 3;
                }
            }

            IF_ICMPLE => {
                let offset = i16::from_be_bytes([code[*pc + 1], code[*pc + 2]]);
                let v2 = frame.pop_int()?;
                let v1 = frame.pop_int()?;
                if v1 <= v2 {
                    *pc = (*pc as i32 + offset as i32) as usize;
                } else {
                    *pc += 3;
                }
            }

            // 无条件跳转
            GOTO => {
                let offset = i16::from_be_bytes([code[*pc + 1], code[*pc + 2]]);
                *pc = (*pc as i32 + offset as i32) as usize;
            }

            // ==================== 返回指令 ====================
            IRETURN => {
                // 弹出返回值
                let return_value = frame.pop()?;
                return Ok(InstructionControl::Return(Some(return_value)));
            }

            RETURN => {
                // void返回
                return Ok(InstructionControl::Return(None));
            }
            _ => {
                return Err(anyhow!("Unknown opcode: 0x{:02X} at pc {}", opcode, *pc));
            }
        }

        Ok(InstructionControl::Continue) // 默认继续执行
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
