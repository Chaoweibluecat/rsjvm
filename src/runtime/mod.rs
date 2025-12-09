//! # 运行时数据区
//!
//! JVM运行时数据区包括：
//! - 程序计数器（PC Register）
//! - Java虚拟机栈（JVM Stack）
//! - 本地方法栈（Native Method Stack）
//! - 堆（Heap）
//! - 方法区（Method Area）
//!
//! ## 学习要点
//! - 栈是线程私有的，每个线程有自己的栈
//! - 堆是线程共享的，所有对象都在堆上分配
//! - 方法区存储类的元数据

pub mod frame;
pub mod heap;
pub mod thread;

pub use frame::Frame;
pub use heap::Heap;
pub use thread::JvmThread;
