//! # JVM线程
//!
//! 每个JVM线程都有自己的虚拟机栈和程序计数器。
//!
//! ## 学习要点
//! - 线程私有数据包括：虚拟机栈、本地方法栈、程序计数器
//! - 每个方法调用都会创建一个新的栈帧
//! - 方法返回时弹出栈帧

use super::Frame;
use crate::Result;
use anyhow::anyhow;

/// JVM线程
#[derive(Debug)]
pub struct JvmThread {
    /// 虚拟机栈（栈帧列表）
    stack: Vec<Frame>,
}

impl JvmThread {
    /// 创建新线程
    pub fn new() -> Self {
        JvmThread { stack: Vec::new() }
    }

    /// 压入新的栈帧
    pub fn push_frame(&mut self, frame: Frame) {
        self.stack.push(frame);
    }

    /// 弹出栈帧
    pub fn pop_frame(&mut self) -> Result<Frame> {
        self.stack
            .pop()
            .ok_or_else(|| anyhow!("Stack is empty"))
    }

    /// 获取当前栈帧
    pub fn current_frame(&self) -> Result<&Frame> {
        self.stack.last().ok_or_else(|| anyhow!("Stack is empty"))
    }

    /// 获取当前栈帧（可变）
    pub fn current_frame_mut(&mut self) -> Result<&mut Frame> {
        self.stack
            .last_mut()
            .ok_or_else(|| anyhow!("Stack is empty"))
    }

    /// 获取栈深度
    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }
}

impl Default for JvmThread {
    fn default() -> Self {
        Self::new()
    }
}
