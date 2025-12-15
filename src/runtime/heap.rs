//! # 堆内存
//!
//! 堆是JVM中用于分配对象的内存区域。
//!
//! ## 学习要点
//! - 所有对象实例和数组都在堆上分配
//! - 堆是垃圾回收的主要区域
//! - 堆是线程共享的
//!
//! ## 简化设计
//! 这个实现使用简单的向量来模拟堆，实际JVM的堆管理要复杂得多

use crate::runtime::frame::JvmValue;
use crate::Result;
use anyhow::{anyhow, Ok};
use std::collections::HashMap;

/// 对象实例
#[derive(Debug, Clone)]
pub struct Object {
    /// 类名
    pub class_name: String,
    /// 字段值
    pub fields: HashMap<String, crate::runtime::frame::JvmValue>,
}

/// 堆
#[derive(Debug)]
pub struct Heap {
    /// 对象存储（使用索引作为对象引用）
    objects: Vec<Option<Object>>,
    /// 空闲列表（已回收的对象索引）
    free_list: Vec<usize>,
}

impl Heap {
    /// 创建新的堆
    pub fn new() -> Self {
        Heap {
            objects: Vec::new(),
            free_list: Vec::new(),
        }
    }

    /// 分配对象
    pub fn allocate(&mut self, class_name: String) -> usize {
        let obj = Object {
            class_name,
            fields: HashMap::new(),
        };

        // 尝试从空闲列表中获取索引
        if let Some(index) = self.free_list.pop() {
            self.objects[index] = Some(obj);
            index
        } else {
            // 否则添加到末尾
            let index = self.objects.len();
            self.objects.push(Some(obj));
            index
        }
    }

    pub fn set_field(&mut self, index: usize, name: String, value: JvmValue) -> Result<()> {
        self.get_mut(index)?.fields.insert(name, value);
        Ok(())
    }

    pub fn get_field(&self, index: usize, name: &String) -> Result<JvmValue> {
        self.get(index)?
            .fields
            .get(name)
            .ok_or(anyhow!("Field not found"))
            .map(|v| v.clone())
    }

    /// 获取对象
    pub fn get(&self, index: usize) -> Result<&Object> {
        self.objects
            .get(index)
            .and_then(|opt| opt.as_ref())
            .ok_or_else(|| anyhow!("Invalid object reference: {}", index))
    }

    /// 获取可变对象
    pub fn get_mut(&mut self, index: usize) -> Result<&mut Object> {
        self.objects
            .get_mut(index)
            .and_then(|opt| opt.as_mut())
            .ok_or_else(|| anyhow!("Invalid object reference: {}", index))
    }

    /// 释放对象（GC使用）
    pub fn free(&mut self, index: usize) -> Result<()> {
        if index >= self.objects.len() {
            return Err(anyhow!("Invalid object reference: {}", index));
        }
        self.objects[index] = None;
        self.free_list.push(index);
        Ok(())
    }

    /// 获取堆中的对象数量
    pub fn object_count(&self) -> usize {
        self.objects.iter().filter(|o| o.is_some()).count()
    }
}

impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}
