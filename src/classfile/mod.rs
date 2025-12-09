//! # Class文件解析模块
//!
//! 这个模块负责解析Java的.class文件，这是理解JVM的第一步。
//!
//! ## Class文件结构
//!
//! ```text
//! ClassFile {
//!     u4             magic;           // 魔数 0xCAFEBABE
//!     u2             minor_version;   // 次版本号
//!     u2             major_version;   // 主版本号
//!     u2             constant_pool_count;
//!     cp_info        constant_pool[constant_pool_count-1];
//!     u2             access_flags;
//!     u2             this_class;
//!     u2             super_class;
//!     u2             interfaces_count;
//!     u2             interfaces[interfaces_count];
//!     u2             fields_count;
//!     field_info     fields[fields_count];
//!     u2             methods_count;
//!     method_info    methods[methods_count];
//!     u2             attributes_count;
//!     attribute_info attributes[attributes_count];
//! }
//! ```

pub mod parser;
pub mod constant_pool;
pub mod attribute;

use crate::Result;
use std::path::Path;

/// Class文件的主结构
#[derive(Debug)]
pub struct ClassFile {
    /// 魔数，应该是0xCAFEBABE
    pub magic: u32,
    /// 次版本号
    pub minor_version: u16,
    /// 主版本号（52 = Java 8）
    pub major_version: u16,
    /// 常量池
    pub constant_pool: constant_pool::ConstantPool,
    /// 访问标志
    pub access_flags: u16,
    /// 当前类索引
    pub this_class: u16,
    /// 父类索引
    pub super_class: u16,
    /// 接口索引表
    pub interfaces: Vec<u16>,
    /// 字段表
    pub fields: Vec<FieldInfo>,
    /// 方法表
    pub methods: Vec<MethodInfo>,
    /// 属性表
    pub attributes: Vec<attribute::AttributeInfo>,
}

/// 字段信息
#[derive(Debug)]
pub struct FieldInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<attribute::AttributeInfo>,
}

/// 方法信息
#[derive(Debug)]
pub struct MethodInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<attribute::AttributeInfo>,
}

/// 访问标志常量
pub mod access_flags {
    pub const ACC_PUBLIC: u16 = 0x0001;
    pub const ACC_PRIVATE: u16 = 0x0002;
    pub const ACC_PROTECTED: u16 = 0x0004;
    pub const ACC_STATIC: u16 = 0x0008;
    pub const ACC_FINAL: u16 = 0x0010;
    pub const ACC_SUPER: u16 = 0x0020;
    pub const ACC_SYNCHRONIZED: u16 = 0x0020;
    pub const ACC_VOLATILE: u16 = 0x0040;
    pub const ACC_BRIDGE: u16 = 0x0040;
    pub const ACC_TRANSIENT: u16 = 0x0080;
    pub const ACC_VARARGS: u16 = 0x0080;
    pub const ACC_NATIVE: u16 = 0x0100;
    pub const ACC_INTERFACE: u16 = 0x0200;
    pub const ACC_ABSTRACT: u16 = 0x0400;
    pub const ACC_STRICT: u16 = 0x0800;
    pub const ACC_SYNTHETIC: u16 = 0x1000;
    pub const ACC_ANNOTATION: u16 = 0x2000;
    pub const ACC_ENUM: u16 = 0x4000;
}

impl ClassFile {
    /// 从文件路径加载class文件
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        parser::parse_class_file(&bytes)
    }

    /// 从字节数组解析class文件
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        parser::parse_class_file(bytes)
    }

    /// 获取类名
    pub fn get_class_name(&self) -> Result<String> {
        self.constant_pool.get_class_name(self.this_class)
    }

    /// 获取父类名
    pub fn get_super_class_name(&self) -> Result<String> {
        if self.super_class == 0 {
            Ok("java/lang/Object".to_string())
        } else {
            self.constant_pool.get_class_name(self.super_class)
        }
    }

    /// 获取Java版本
    pub fn get_java_version(&self) -> String {
        match self.major_version {
            52 => "Java 8".to_string(),
            53 => "Java 9".to_string(),
            54 => "Java 10".to_string(),
            55 => "Java 11".to_string(),
            _ => format!("Java (version {})", self.major_version),
        }
    }
}
