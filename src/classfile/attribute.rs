//! # 属性信息
//!
//! 属性（Attribute）用于描述class文件、字段、方法等的附加信息。
//! 最重要的属性是Code属性，它包含了方法的字节码指令。
//!
//! ## 常见属性
//! - Code: 方法的字节码
//! - SourceFile: 源文件名
//! - LineNumberTable: 行号表
//! - LocalVariableTable: 局部变量表

use crate::Result;
use anyhow::Context;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

/// 属性信息（简化版）
#[derive(Debug)]
pub struct AttributeInfo {
    pub name_index: u16,
    pub info: Vec<u8>,
}

/// Code属性（方法的字节码）
#[derive(Debug)]
pub struct CodeAttribute {
    /// 操作数栈的最大深度
    pub max_stack: u16,
    /// 局部变量表的大小
    pub max_locals: u16,
    /// 字节码指令
    pub code: Vec<u8>,
    /// 异常表
    pub exception_table: Vec<ExceptionHandler>,
    /// 属性表
    pub attributes: Vec<AttributeInfo>,
}

/// 异常处理器
#[derive(Debug)]
pub struct ExceptionHandler {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

impl AttributeInfo {
    /// 解析为Code属性
    pub fn parse_code_attribute(&self) -> Result<CodeAttribute> {
        let mut reader = Cursor::new(&self.info);

        let max_stack = reader
            .read_u16::<BigEndian>()
            .context("Failed to read max_stack")?;
        let max_locals = reader
            .read_u16::<BigEndian>()
            .context("Failed to read max_locals")?;

        let code_length = reader
            .read_u32::<BigEndian>()
            .context("Failed to read code_length")?;
        let mut code = vec![0u8; code_length as usize];
        std::io::Read::read_exact(&mut reader, &mut code)?;

        let exception_table_length = reader.read_u16::<BigEndian>()?;
        let mut exception_table = Vec::with_capacity(exception_table_length as usize);
        for _ in 0..exception_table_length {
            exception_table.push(ExceptionHandler {
                start_pc: reader.read_u16::<BigEndian>()?,
                end_pc: reader.read_u16::<BigEndian>()?,
                handler_pc: reader.read_u16::<BigEndian>()?,
                catch_type: reader.read_u16::<BigEndian>()?,
            });
        }

        let attributes_count = reader.read_u16::<BigEndian>()?;
        let mut attributes = Vec::with_capacity(attributes_count as usize);
        for _ in 0..attributes_count {
            let name_index = reader.read_u16::<BigEndian>()?;
            let length = reader.read_u32::<BigEndian>()?;
            let mut info = vec![0u8; length as usize];
            std::io::Read::read_exact(&mut reader, &mut info)?;
            attributes.push(AttributeInfo { name_index, info });
        }

        Ok(CodeAttribute {
            max_stack,
            max_locals,
            code,
            exception_table,
            attributes,
        })
    }
}
