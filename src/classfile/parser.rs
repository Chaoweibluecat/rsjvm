//! # Class文件解析器
//!
//! 这个模块负责从字节数组中解析出ClassFile结构。
//!
//! ## 学习要点
//! - Java class文件使用大端字节序（Big-Endian）
//! - 需要按照JVM规范的顺序依次读取各个部分
//! - 错误处理很重要，要能够识别无效的class文件

use super::*;
use crate::Result;
use anyhow::{anyhow, Context};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

/// Class文件魔数
const MAGIC: u32 = 0xCAFEBABE;

/// 解析class文件
pub fn parse_class_file(bytes: &[u8]) -> Result<ClassFile> {
    let mut reader = Cursor::new(bytes);

    // 1. 读取魔数
    let magic = reader
        .read_u32::<BigEndian>()
        .context("Failed to read magic number")?;
    if magic != MAGIC {
        return Err(anyhow!("Invalid magic number: 0x{:X}", magic));
    }

    // 2. 读取版本号
    let minor_version = reader
        .read_u16::<BigEndian>()
        .context("Failed to read minor version")?;
    let major_version = reader
        .read_u16::<BigEndian>()
        .context("Failed to read major version")?;

    // 3. 解析常量池
    let constant_pool = parse_constant_pool(&mut reader)?;

    // 4. 读取访问标志
    let access_flags = reader
        .read_u16::<BigEndian>()
        .context("Failed to read access flags")?;

    // 5. 读取类索引
    let this_class = reader
        .read_u16::<BigEndian>()
        .context("Failed to read this_class")?;
    let super_class = reader
        .read_u16::<BigEndian>()
        .context("Failed to read super_class")?;

    // 6. 读取接口
    let interfaces = parse_interfaces(&mut reader)?;

    // 7. 读取字段
    let fields = parse_fields(&mut reader, &constant_pool)?;

    // 8. 读取方法
    let methods = parse_methods(&mut reader, &constant_pool)?;

    // 9. 读取属性
    let attributes = parse_attributes(&mut reader, &constant_pool)?;

    Ok(ClassFile {
        magic,
        minor_version,
        major_version,
        constant_pool,
        access_flags,
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attributes,
    })
}

/// 解析常量池
fn parse_constant_pool(reader: &mut Cursor<&[u8]>) -> Result<constant_pool::ConstantPool> {
    let count = reader
        .read_u16::<BigEndian>()
        .context("Failed to read constant pool count")?;

    let mut pool = constant_pool::ConstantPool::new(count as usize);

    let mut i = 1;
    while i < count {
        let tag = reader
            .read_u8()
            .context(format!("Failed to read constant pool tag at {}", i))?;

        use constant_pool::tags::*;
        use constant_pool::ConstantPoolEntry;

        let entry = match tag {
            CONSTANT_UTF8 => {
                let length = reader.read_u16::<BigEndian>()?;
                let mut buf = vec![0u8; length as usize];
                std::io::Read::read_exact(reader, &mut buf)?;
                // Java使用修改过的UTF-8编码，这里简化处理
                let s = String::from_utf8(buf)
                    .context(format!("Invalid UTF-8 at constant pool index {}", i))?;
                ConstantPoolEntry::Utf8(s)
            }
            CONSTANT_INTEGER => {
                let value = reader.read_i32::<BigEndian>()?;
                ConstantPoolEntry::Integer(value)
            }
            CONSTANT_FLOAT => {
                let value = reader.read_f32::<BigEndian>()?;
                ConstantPoolEntry::Float(value)
            }
            CONSTANT_LONG => {
                let value = reader.read_i64::<BigEndian>()?;
                pool.set(i, ConstantPoolEntry::Long(value));
                i += 1; // Long占两个位置
                continue;
            }
            CONSTANT_DOUBLE => {
                let value = reader.read_f64::<BigEndian>()?;
                pool.set(i, ConstantPoolEntry::Double(value));
                i += 1; // Double占两个位置
                continue;
            }
            CONSTANT_CLASS => {
                let name_index = reader.read_u16::<BigEndian>()?;
                ConstantPoolEntry::Class { name_index }
            }
            CONSTANT_STRING => {
                let string_index = reader.read_u16::<BigEndian>()?;
                ConstantPoolEntry::String { string_index }
            }
            CONSTANT_FIELDREF => {
                let class_index = reader.read_u16::<BigEndian>()?;
                let name_and_type_index = reader.read_u16::<BigEndian>()?;
                ConstantPoolEntry::FieldRef {
                    class_index,
                    name_and_type_index,
                }
            }
            CONSTANT_METHODREF => {
                let class_index = reader.read_u16::<BigEndian>()?;
                let name_and_type_index = reader.read_u16::<BigEndian>()?;
                ConstantPoolEntry::MethodRef {
                    class_index,
                    name_and_type_index,
                }
            }
            CONSTANT_INTERFACE_METHODREF => {
                let class_index = reader.read_u16::<BigEndian>()?;
                let name_and_type_index = reader.read_u16::<BigEndian>()?;
                ConstantPoolEntry::InterfaceMethodRef {
                    class_index,
                    name_and_type_index,
                }
            }
            CONSTANT_NAME_AND_TYPE => {
                let name_index = reader.read_u16::<BigEndian>()?;
                let descriptor_index = reader.read_u16::<BigEndian>()?;
                ConstantPoolEntry::NameAndType {
                    name_index,
                    descriptor_index,
                }
            }
            CONSTANT_METHOD_HANDLE => {
                let reference_kind = reader.read_u8()?;
                let reference_index = reader.read_u16::<BigEndian>()?;
                ConstantPoolEntry::MethodHandle {
                    reference_kind,
                    reference_index,
                }
            }
            CONSTANT_METHOD_TYPE => {
                let descriptor_index = reader.read_u16::<BigEndian>()?;
                ConstantPoolEntry::MethodType { descriptor_index }
            }
            CONSTANT_INVOKE_DYNAMIC => {
                let bootstrap_method_attr_index = reader.read_u16::<BigEndian>()?;
                let name_and_type_index = reader.read_u16::<BigEndian>()?;
                ConstantPoolEntry::InvokeDynamic {
                    bootstrap_method_attr_index,
                    name_and_type_index,
                }
            }
            _ => return Err(anyhow!("Unknown constant pool tag: {}", tag)),
        };

        pool.set(i, entry);
        i += 1;
    }

    Ok(pool)
}

/// 解析接口表
fn parse_interfaces(reader: &mut Cursor<&[u8]>) -> Result<Vec<u16>> {
    let count = reader.read_u16::<BigEndian>()?;
    let mut interfaces = Vec::with_capacity(count as usize);
    for _ in 0..count {
        interfaces.push(reader.read_u16::<BigEndian>()?);
    }
    Ok(interfaces)
}

/// 解析字段表
fn parse_fields(
    reader: &mut Cursor<&[u8]>,
    pool: &constant_pool::ConstantPool,
) -> Result<Vec<FieldInfo>> {
    let count = reader.read_u16::<BigEndian>()?;
    let mut fields = Vec::with_capacity(count as usize);
    for _ in 0..count {
        fields.push(parse_field(reader, pool)?);
    }
    Ok(fields)
}

/// 解析单个字段
fn parse_field(
    reader: &mut Cursor<&[u8]>,
    pool: &constant_pool::ConstantPool,
) -> Result<FieldInfo> {
    let access_flags = reader.read_u16::<BigEndian>()?;
    let name_index = reader.read_u16::<BigEndian>()?;
    let descriptor_index = reader.read_u16::<BigEndian>()?;
    let attributes = parse_attributes(reader, pool)?;

    Ok(FieldInfo {
        access_flags,
        name_index,
        descriptor_index,
        attributes,
    })
}

/// 解析方法表
fn parse_methods(
    reader: &mut Cursor<&[u8]>,
    pool: &constant_pool::ConstantPool,
) -> Result<Vec<MethodInfo>> {
    let count = reader.read_u16::<BigEndian>()?;
    let mut methods = Vec::with_capacity(count as usize);
    for _ in 0..count {
        methods.push(parse_method(reader, pool)?);
    }
    Ok(methods)
}

/// 解析单个方法
fn parse_method(
    reader: &mut Cursor<&[u8]>,
    pool: &constant_pool::ConstantPool,
) -> Result<MethodInfo> {
    let access_flags = reader.read_u16::<BigEndian>()?;
    let name_index = reader.read_u16::<BigEndian>()?;
    let descriptor_index = reader.read_u16::<BigEndian>()?;
    let attributes = parse_attributes(reader, pool)?;

    Ok(MethodInfo {
        access_flags,
        name_index,
        descriptor_index,
        attributes,
    })
}

/// 解析属性表
fn parse_attributes(
    reader: &mut Cursor<&[u8]>,
    pool: &constant_pool::ConstantPool,
) -> Result<Vec<attribute::AttributeInfo>> {
    let count = reader.read_u16::<BigEndian>()?;
    let mut attributes = Vec::with_capacity(count as usize);
    for _ in 0..count {
        attributes.push(parse_attribute(reader, pool)?);
    }
    Ok(attributes)
}

/// 解析单个属性
fn parse_attribute(
    reader: &mut Cursor<&[u8]>,
    _pool: &constant_pool::ConstantPool,
) -> Result<attribute::AttributeInfo> {
    let name_index = reader.read_u16::<BigEndian>()?;
    let length = reader.read_u32::<BigEndian>()?;
    let mut info = vec![0u8; length as usize];
    std::io::Read::read_exact(reader, &mut info)?;

    Ok(attribute::AttributeInfo { name_index, info })
}
