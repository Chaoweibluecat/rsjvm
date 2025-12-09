//! # RSJVM - Rust实现的学习型JVM
//!
//! 这是一个用于学习Java虚拟机原理的项目，使用Rust实现了JVM的核心功能。
//!
//! ## 模块结构
//!
//! - `classfile`: Class文件解析，理解字节码结构
//! - `runtime`: 运行时数据区，包括栈帧、堆、方法区
//! - `interpreter`: 字节码解释器，执行指令
//! - `classloader`: 类加载器，负责加载class文件
//! - `gc`: 垃圾回收器（简化版）

pub mod classfile;
pub mod runtime;
pub mod interpreter;
pub mod classloader;
pub mod gc;

/// 通用错误类型
pub type Result<T> = anyhow::Result<T>;
