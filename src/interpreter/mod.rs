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
use crate::runtime::{Frame, Heap, JvmThread};
use crate::runtime::frame::JvmValue;
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
    /// 已加载的类
    pub classes: std::collections::HashMap<String, ClassFile>,
}

impl Interpreter {
    /// 创建新的解释器
    pub fn new() -> Self {
        Interpreter {
            heap: Heap::new(),
            thread: JvmThread::new(),
            classes: std::collections::HashMap::new(),
        }
    }

    /// 执行方法
    /// 返回方法的返回值（如果有）
    pub fn execute_method(&mut self, code: &[u8], max_locals: usize, max_stack: usize) -> Result<Option<crate::runtime::frame::JvmValue>> {
        let mut frame = Frame::new(max_locals, max_stack);
        let mut return_value = None;

        while frame.pc < code.len() {
            let opcode = code[frame.pc];
            let control = self.execute_instruction(opcode, code, &mut frame)?;
            match control {
                InstructionControl::Continue => {},
                InstructionControl::Return(val) => {
                    return_value = val;
                    break;
                }
            }
        }

        Ok(return_value)
    }

    /// 执行单条指令
    /// 返回指令控制信息
    fn execute_instruction(&mut self, opcode: u8, code: &[u8], frame: &mut Frame) -> Result<InstructionControl> {
        use instructions::opcodes::*;

        match opcode {
            NOP => {
                // 无操作
                frame.pc += 1;
            }

            // ==================== 常量指令 ====================
            ICONST_M1 => {
                frame.push(crate::runtime::frame::JvmValue::Int(-1));
                frame.pc += 1;
            }
            ICONST_0 => {
                frame.push(crate::runtime::frame::JvmValue::Int(0));
                frame.pc += 1;
            }
            ICONST_1 => {
                frame.push(crate::runtime::frame::JvmValue::Int(1));
                frame.pc += 1;
            }
            ICONST_2 => {
                frame.push(crate::runtime::frame::JvmValue::Int(2));
                frame.pc += 1;
            }
            ICONST_3 => {
                frame.push(crate::runtime::frame::JvmValue::Int(3));
                frame.pc += 1;
            }
            ICONST_4 => {
                frame.push(crate::runtime::frame::JvmValue::Int(4));
                frame.pc += 1;
            }
            ICONST_5 => {
                frame.push(crate::runtime::frame::JvmValue::Int(5));
                frame.pc += 1;
            }

            BIPUSH => {
                let value = code[frame.pc + 1] as i8;
                frame.push(crate::runtime::frame::JvmValue::Int(value as i32));
                frame.pc += 2;
            }

            SIPUSH => {
                let value = i16::from_be_bytes([code[frame.pc + 1], code[frame.pc + 2]]);
                frame.push(crate::runtime::frame::JvmValue::Int(value as i32));
                frame.pc += 3;
            }

            // ==================== 加载指令 ====================
            ILOAD_0 | ILOAD_1 | ILOAD_2 | ILOAD_3 => {
                let index = (opcode - ILOAD_0) as usize;
                let value = frame.get_local(index)?.clone();
                frame.push(value);
                frame.pc += 1;
            }

            // ==================== 存储指令 ====================
            ISTORE_0 | ISTORE_1 | ISTORE_2 | ISTORE_3 => {
                let index = (opcode - ISTORE_0) as usize;
                let value = frame.pop()?;
                frame.set_local(index, value)?;
                frame.pc += 1;
            }

            // ==================== 运算指令 ====================
            IADD => {
                let v2 = frame.pop_int()?;
                let v1 = frame.pop_int()?;
                frame.push(crate::runtime::frame::JvmValue::Int(v1 + v2));
                frame.pc += 1;
            }

            ISUB => {
                let v2 = frame.pop_int()?;
                let v1 = frame.pop_int()?;
                frame.push(crate::runtime::frame::JvmValue::Int(v1 - v2));
                frame.pc += 1;
            }

            IMUL => {
                let v2 = frame.pop_int()?;
                let v1 = frame.pop_int()?;
                frame.push(crate::runtime::frame::JvmValue::Int(v1 * v2));
                frame.pc += 1;
            }

            IDIV => {
                let v2 = frame.pop_int()?;
                let v1 = frame.pop_int()?;
                if v2 == 0 {
                    return Err(anyhow!("Division by zero"));
                }
                frame.push(crate::runtime::frame::JvmValue::Int(v1 / v2));
                frame.pc += 1;
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
                return Err(anyhow!("Unknown opcode: 0x{:02X} at pc {}", opcode, frame.pc));
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
