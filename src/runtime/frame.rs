//! # 栈帧
//!
//! 栈帧（Frame）是JVM栈的基本单位，每个方法调用都会创建一个新的栈帧。
//!
//! ## 栈帧结构
//! - 局部变量表：存储方法参数和局部变量
//! - 操作数栈：执行字节码指令时的工作区
//! - 常量池引用：指向当前类的常量池
//!
//! ## 学习要点
//! - 局部变量表的大小在编译时确定
//! - 操作数栈用于计算和传递参数
//! - JVM是基于栈的虚拟机

use crate::Result;
use anyhow::anyhow;

/// JVM值类型
#[derive(Debug, Clone)]
pub enum JvmValue {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Reference(Option<usize>), // 对象引用（堆上的索引）
}

/// 栈帧
#[derive(Debug)]
pub struct Frame {
    /// 局部变量表
    local_vars: Vec<JvmValue>,
    /// 操作数栈
    operand_stack: Vec<JvmValue>,

    /// 动态链接 - 指向当前方法所属类的名称
    /// 用于解析符号引用
    pub class_name: String,

    /// 返回地址 - 方法正常返回后的指令位置（在调用者中的PC）
    pub return_address: Option<usize>,

    /// 当前方法的字节码
    /// 注意：这里使用 Vec 而不是引用，简化生命周期管理
    pub code: Vec<u8>,

    /// 操作数栈最大深度（用于调试）
    pub max_stack: usize,
    /// 局部变量表大小（用于调试）
    pub max_locals: usize,
}

impl Frame {
    /// 创建新的栈帧
    pub fn new(max_locals: usize, max_stack: usize) -> Self {
        Frame {
            local_vars: vec![JvmValue::Int(0); max_locals],
            operand_stack: Vec::with_capacity(max_stack),
            class_name: String::new(),  // 稍后设置
            return_address: None,
            code: Vec::new(),  // 稍后设置
            max_stack,
            max_locals,
        }
    }

    /// 创建带完整信息的栈帧
    pub fn new_with_context(
        max_locals: usize,
        max_stack: usize,
        class_name: String,
        code: Vec<u8>,
        return_address: Option<usize>,
    ) -> Self {
        Frame {
            local_vars: vec![JvmValue::Int(0); max_locals],
            operand_stack: Vec::with_capacity(max_stack),
            class_name,
            return_address,
            code,
            max_stack,
            max_locals,
        }
    }

    // ==================== 局部变量表操作 ====================

    /// 获取局部变量
    pub fn get_local(&self, index: usize) -> Result<&JvmValue> {
        self.local_vars
            .get(index)
            .ok_or_else(|| anyhow!("Local variable index out of bounds: {}", index))
    }

    /// 设置局部变量
    pub fn set_local(&mut self, index: usize, value: JvmValue) -> Result<()> {
        if index >= self.local_vars.len() {
            return Err(anyhow!("Local variable index out of bounds: {}", index));
        }
        self.local_vars[index] = value;
        Ok(())
    }

    // ==================== 操作数栈操作 ====================

    /// 压栈
    pub fn push(&mut self, value: JvmValue) {
        self.operand_stack.push(value);
    }

    /// 弹栈
    pub fn pop(&mut self) -> Result<JvmValue> {
        self.operand_stack
            .pop()
            .ok_or_else(|| anyhow!("Operand stack is empty"))
    }

    /// 查看栈顶元素（不弹出）
    pub fn peek(&self) -> Result<&JvmValue> {
        self.operand_stack
            .last()
            .ok_or_else(|| anyhow!("Operand stack is empty"))
    }

    /// 弹出int值
    pub fn pop_int(&mut self) -> Result<i32> {
        match self.pop()? {
            JvmValue::Int(val) => Ok(val),
            _ => Err(anyhow!("Expected Int on stack")),
        }
    }

    /// 弹出long值
    pub fn pop_long(&mut self) -> Result<i64> {
        match self.pop()? {
            JvmValue::Long(val) => Ok(val),
            _ => Err(anyhow!("Expected Long on stack")),
        }
    }

    /// 弹出float值
    pub fn pop_float(&mut self) -> Result<f32> {
        match self.pop()? {
            JvmValue::Float(val) => Ok(val),
            _ => Err(anyhow!("Expected Float on stack")),
        }
    }

    /// 弹出double值
    pub fn pop_double(&mut self) -> Result<f64> {
        match self.pop()? {
            JvmValue::Double(val) => Ok(val),
            _ => Err(anyhow!("Expected Double on stack")),
        }
    }

    /// 弹出引用
    pub fn pop_ref(&mut self) -> Result<Option<usize>> {
        match self.pop()? {
            JvmValue::Reference(val) => Ok(val),
            _ => Err(anyhow!("Expected Reference on stack")),
        }
    }

    /// 获取操作数栈大小
    pub fn stack_size(&self) -> usize {
        self.operand_stack.len()
    }
}
