//! # 常量池
//!
//! 常量池是class文件中非常重要的部分，包含了类中使用的所有常量，包括：
//! - 字面量：字符串、整数、浮点数等
//! - 符号引用：类、字段、方法等的引用
//!
//! ## 学习要点
//! - 常量池索引从1开始（0保留）
//! - Long和Double占用两个索引位
//! - 常量池项之间会相互引用

use crate::Result;
use anyhow::anyhow;

/// 常量池
#[derive(Debug)]
pub struct ConstantPool {
    pub entries: Vec<Option<ConstantPoolEntry>>,
}

/// 常量池项
#[derive(Debug, Clone)]
pub enum ConstantPoolEntry {
    /// UTF-8字符串
    Utf8(String),
    /// 整数
    Integer(i32),
    /// 浮点数
    Float(f32),
    /// 长整数
    Long(i64),
    /// 双精度浮点数
    Double(f64),
    /// 类引用
    Class { name_index: u16 },
    /// 字符串引用
    String { string_index: u16 },
    /// 字段引用
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    /// 方法引用
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    /// 接口方法引用
    InterfaceMethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    /// 名称和类型
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    /// 方法句柄
    MethodHandle {
        reference_kind: u8,
        reference_index: u16,
    },
    /// 方法类型
    MethodType { descriptor_index: u16 },
    /// 动态调用
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
}

impl ConstantPool {
    /// 创建新的常量池
    pub fn new(size: usize) -> Self {
        ConstantPool {
            entries: vec![None; size],
        }
    }

    /// 获取常量池项
    pub fn get(&self, index: u16) -> Result<&ConstantPoolEntry> {
        if index == 0 || index as usize >= self.entries.len() {
            return Err(anyhow!("Invalid constant pool index: {}", index));
        }
        self.entries[index as usize]
            .as_ref()
            .ok_or_else(|| anyhow!("Constant pool entry at {} is None", index))
    }

    /// 获取UTF-8字符串
    pub fn get_utf8(&self, index: u16) -> Result<String> {
        match self.get(index)? {
            ConstantPoolEntry::Utf8(s) => Ok(s.clone()),
            _ => Err(anyhow!("Expected Utf8 at index {}", index)),
        }
    }

    /// 获取类名
    pub fn get_class_name(&self, index: u16) -> Result<String> {
        match self.get(index)? {
            ConstantPoolEntry::Class { name_index } => self.get_utf8(*name_index),
            _ => Err(anyhow!("Expected Class at index {}", index)),
        }
    }

    /// 获取名称和类型
    pub fn get_name_and_type(&self, index: u16) -> Result<(String, String)> {
        match self.get(index)? {
            ConstantPoolEntry::NameAndType {
                name_index,
                descriptor_index,
            } => {
                let name = self.get_utf8(*name_index)?;
                let descriptor = self.get_utf8(*descriptor_index)?;
                Ok((name, descriptor))
            }
            _ => Err(anyhow!("Expected NameAndType at index {}", index)),
        }
    }

    /// 设置常量池项
    pub fn set(&mut self, index: u16, entry: ConstantPoolEntry) {
        self.entries[index as usize] = Some(entry);
    }

    /// 调试用：打印常量池的所有内容
    pub fn debug_print(&self) {
        println!("=== 常量池调试输出 ===");
        println!("总大小: {}", self.entries.len());
        for (i, entry) in self.entries.iter().enumerate() {
            if i == 0 {
                continue;
            }
            match entry {
                Some(e) => println!("[{:2}] {:?}", i, e),
                None => println!("[{:2}] <None>", i),
            }
        }
        println!("======================");
    }
}

/// 常量池标签常量
pub mod tags {
    pub const CONSTANT_UTF8: u8 = 1;
    pub const CONSTANT_INTEGER: u8 = 3;
    pub const CONSTANT_FLOAT: u8 = 4;
    pub const CONSTANT_LONG: u8 = 5;
    pub const CONSTANT_DOUBLE: u8 = 6;
    pub const CONSTANT_CLASS: u8 = 7;
    pub const CONSTANT_STRING: u8 = 8;
    pub const CONSTANT_FIELDREF: u8 = 9;
    pub const CONSTANT_METHODREF: u8 = 10;
    pub const CONSTANT_INTERFACE_METHODREF: u8 = 11;
    pub const CONSTANT_NAME_AND_TYPE: u8 = 12;
    pub const CONSTANT_METHOD_HANDLE: u8 = 15;
    pub const CONSTANT_METHOD_TYPE: u8 = 16;
    pub const CONSTANT_INVOKE_DYNAMIC: u8 = 18;
}
