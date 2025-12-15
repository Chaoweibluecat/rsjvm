//! # 方法区 (Metaspace)
//!
//! 方法区是JVM运行时数据区的一部分，存储已加载类的元数据信息。
//! 在Java 8+中，称为Metaspace，替代了之前的永久代(PermGen)。
//!
//! ## 主要职责
//! - 存储类的结构信息（字段、方法、常量池等）
//! - 管理运行时常量池
//! - 提供符号引用到直接引用的解析
//!
//! ## 学习要点
//! - 方法区是所有线程共享的
//! - 类的元数据在首次使用时加载
//! - 常量池解析采用延迟解析策略

use crate::classfile::constant_pool::ConstantPoolEntry;
use crate::classfile::{access_flags, ClassFile, MethodInfo};
use crate::Result;
use anyhow::anyhow;
use std::collections::HashMap;

/// 方法区 - 存储所有已加载类的元数据
#[derive(Debug)]
pub struct Metaspace {
    /// 所有已加载的类
    /// Key: 完全限定类名 (如 "java/lang/Object", "com/example/MyClass")
    classes: HashMap<String, ClassMetadata>,
}

/// 类元数据 - 运行时类的表示
#[derive(Debug)]
pub struct ClassMetadata {
    /// 类名
    pub name: String,

    /// 父类名
    pub super_class: Option<String>,

    /// 接口列表
    pub interfaces: Vec<String>,

    /// 访问标志
    pub access_flags: u16,

    /// 原始常量池（来自ClassFile）
    pub constant_pool: Vec<Option<ConstantPoolEntry>>,

    /// 运行时常量池 - 符号引用解析缓存
    pub runtime_pool: RuntimeConstantPool,

    /// 方法表 - 快速查找方法
    /// Key: "方法名:方法描述符" (如 "add:(II)I")
    pub methods: HashMap<String, MethodMetadata>,

    /// 字段表 - 快速查找字段
    /// Key: "字段名:字段描述符" (如 "count:I")
    pub fields: HashMap<String, FieldMetadata>,

    /// 静态字段的值存储
    pub static_fields: HashMap<String, crate::runtime::frame::JvmValue>,

    /// 类初始化状态
    pub state: ClassState,
}

/// 类初始化状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClassState {
    /// 已加载 - class文件已读取并解析
    Loaded,
    /// 已链接 - 验证、准备、解析完成
    Linked,
    /// 初始化中 - 正在执行<clinit>方法
    Initializing,
    /// 已初始化 - 类已经可以使用
    Initialized,
}

/// 运行时常量池 - 缓存已解析的符号引用
#[derive(Debug)]
pub struct RuntimeConstantPool {
    /// 已解析的方法引用
    /// Key: 常量池索引, Value: 解析后的方法信息
    pub resolved_methods: HashMap<u16, ResolvedMethodRef>,

    /// 已解析的字段引用
    /// Key: 常量池索引, Value: 解析后的字段信息
    pub resolved_fields: HashMap<u16, ResolvedFieldRef>,

    /// 已解析的类引用
    /// Key: 常量池索引, Value: 类名
    pub resolved_classes: HashMap<u16, String>,
}

/// 已解析的方法引用
#[derive(Debug, Clone)]
pub struct ResolvedMethodRef {
    /// 方法所在的类名
    pub class_name: String,
    /// 方法名
    pub method_name: String,
    /// 方法描述符
    pub descriptor: String,
}

/// 已解析的字段引用
#[derive(Debug, Clone)]
pub struct ResolvedFieldRef {
    /// 字段所在的类名
    pub class_name: String,
    /// 字段名
    pub field_name: String,
    /// 字段描述符
    pub descriptor: String,
}

/// 方法元数据
#[derive(Debug, Clone)]
pub struct MethodMetadata {
    /// 方法名
    pub name: String,
    /// 方法描述符 (如 "(II)I" 表示 int add(int, int))
    pub descriptor: String,
    /// 访问标志
    pub access_flags: u16,
    /// 操作数栈最大深度
    pub max_stack: usize,
    /// 局部变量表大小
    pub max_locals: usize,
    /// 字节码
    pub code: Vec<u8>,
    /// 是否是静态方法
    pub is_static: bool,
    /// 是否是本地方法
    pub is_native: bool,
    /// 是否是抽象方法
    pub is_abstract: bool,
}

/// 字段元数据
#[derive(Debug, Clone)]
pub struct FieldMetadata {
    /// 字段名
    pub name: String,
    /// 字段描述符 (如 "I" 表示 int, "Ljava/lang/String;" 表示 String)
    pub descriptor: String,
    /// 访问标志
    pub access_flags: u16,
    /// 是否是静态字段
    pub is_static: bool,
}

impl Metaspace {
    /// 创建新的方法区
    pub fn new() -> Self {
        Metaspace {
            classes: HashMap::new(),
        }
    }

    /// 加载类
    /// 将ClassFile转换为ClassMetadata并存储
    pub fn load_class(&mut self, class_file: ClassFile) -> Result<()> {
        // 获取类名
        let class_name = class_file.get_class_name()?;

        // 如果类已经加载，跳过
        if self.classes.contains_key(&class_name) {
            return Ok(());
        }

        // 获取父类名
        let super_class = if class_file.super_class == 0 {
            None
        } else {
            Some(class_file.get_super_class_name()?)
        };

        // 获取接口列表
        let mut interfaces = Vec::new();
        for &interface_index in &class_file.interfaces {
            let interface_name = class_file.constant_pool.get_class_name(interface_index)?;
            interfaces.push(interface_name);
        }

        // 解析方法
        let methods = Self::parse_methods(&class_file)?;

        // 解析字段
        let fields = Self::parse_fields(&class_file)?;

        // 创建类元数据
        let metadata = ClassMetadata {
            name: class_name.clone(),
            super_class,
            interfaces,
            access_flags: class_file.access_flags,
            constant_pool: class_file.constant_pool.entries.clone(),
            runtime_pool: RuntimeConstantPool::new(),
            methods,
            fields,
            static_fields: HashMap::new(),
            state: ClassState::Loaded,
        };

        // 存储到方法区
        self.classes.insert(class_name, metadata);

        Ok(())
    }

    /// 解析方法表
    fn parse_methods(class_file: &ClassFile) -> Result<HashMap<String, MethodMetadata>> {
        let mut methods = HashMap::new();

        for method in &class_file.methods {
            let name = class_file.constant_pool.get_utf8(method.name_index)?;
            let descriptor = class_file.constant_pool.get_utf8(method.descriptor_index)?;

            let is_static = (method.access_flags & access_flags::ACC_STATIC) != 0;
            let is_native = (method.access_flags & access_flags::ACC_NATIVE) != 0;
            let is_abstract = (method.access_flags & access_flags::ACC_ABSTRACT) != 0;

            // 查找Code属性
            let (max_stack, max_locals, code) = if is_native || is_abstract {
                // native和abstract方法没有字节码
                (0, 0, Vec::new())
            } else {
                Self::extract_code_from_method(method, class_file)?
            };

            let method_metadata = MethodMetadata {
                name: name.clone(),
                descriptor: descriptor.clone(),
                access_flags: method.access_flags,
                max_stack,
                max_locals,
                code,
                is_static,
                is_native,
                is_abstract,
            };

            // Key格式: "方法名:描述符"
            let key = format!("{}:{}", name, descriptor);
            methods.insert(key, method_metadata);
        }

        Ok(methods)
    }

    /// 从方法属性中提取Code属性
    fn extract_code_from_method(
        method: &MethodInfo,
        class_file: &ClassFile,
    ) -> Result<(usize, usize, Vec<u8>)> {
        for attr in &method.attributes {
            // 检查属性名是否为 "Code"
            let attr_name = class_file.constant_pool.get_utf8(attr.name_index)?;
            if attr_name == "Code" {
                // 解析Code属性
                let code_attr = attr.parse_code_attribute()?;
                return Ok((
                    code_attr.max_stack as usize,
                    code_attr.max_locals as usize,
                    code_attr.code.clone(),
                ));
            }
        }
        Err(anyhow!(
            "Method {}:{} has no Code attribute",
            class_file.constant_pool.get_utf8(method.name_index)?,
            class_file.constant_pool.get_utf8(method.descriptor_index)?
        ))
    }

    /// 解析字段表
    fn parse_fields(class_file: &ClassFile) -> Result<HashMap<String, FieldMetadata>> {
        let mut fields = HashMap::new();

        for field in &class_file.fields {
            let name = class_file.constant_pool.get_utf8(field.name_index)?;
            let descriptor = class_file.constant_pool.get_utf8(field.descriptor_index)?;
            let is_static = (field.access_flags & access_flags::ACC_STATIC) != 0;

            let field_metadata = FieldMetadata {
                name: name.clone(),
                descriptor: descriptor.clone(),
                access_flags: field.access_flags,
                is_static,
            };

            // Key格式: "字段名:描述符"
            let key = format!("{}:{}", name, descriptor);
            fields.insert(key, field_metadata);
        }

        Ok(fields)
    }

    /// 获取类元数据
    pub fn get_class(&self, class_name: &str) -> Result<&ClassMetadata> {
        self.classes
            .get(class_name)
            .ok_or_else(|| anyhow!("Class not found: {}", class_name))
    }

    /// 获取类元数据（可变）
    pub fn get_class_mut(&mut self, class_name: &str) -> Result<&mut ClassMetadata> {
        self.classes
            .get_mut(class_name)
            .ok_or_else(|| anyhow!("Class not found: {}", class_name))
    }

    /// 检查类是否已加载
    pub fn is_class_loaded(&self, class_name: &str) -> bool {
        self.classes.contains_key(class_name)
    }

    /// 获取已加载的类列表
    pub fn loaded_classes(&self) -> Vec<String> {
        self.classes.keys().cloned().collect()
    }
}

impl ClassMetadata {
    /// 查找方法
    /// 如果当前类没有，会递归查找父类（TODO: 后续实现）
    pub fn find_method(&self, name: &str, descriptor: &str) -> Result<&MethodMetadata> {
        let key = format!("{}:{}", name, descriptor);
        self.methods
            .get(&key)
            .ok_or_else(|| anyhow!("Method not found: {}.{}{}", self.name, name, descriptor))
    }

    /// 查找字段
    pub fn find_field(&self, name: &str, descriptor: &str) -> Result<&FieldMetadata> {
        let key = format!("{}:{}", name, descriptor);
        self.fields
            .get(&key)
            .ok_or_else(|| anyhow!("Field not found: {}.{}{}", self.name, name, descriptor))
    }

    /// 解析 NameAndType 条目（辅助方法）
    /// 返回 (name, descriptor) 元组
    fn resolve_name_and_type(&self, index: u16) -> Result<(String, String)> {
        let nat_entry = self
            .constant_pool
            .get(index as usize)
            .ok_or_else(|| anyhow!("Invalid NameAndType index: {}", index))?
            .as_ref()
            .ok_or_else(|| anyhow!("NameAndType entry is None"))?;

        let (name_index, descriptor_index) = match nat_entry {
            ConstantPoolEntry::NameAndType {
                name_index,
                descriptor_index,
            } => (*name_index, *descriptor_index),
            _ => return Err(anyhow!("Expected NameAndType entry")),
        };

        let name = self
            .constant_pool
            .get(name_index as usize)
            .and_then(|e| e.as_ref())
            .and_then(|e| {
                if let ConstantPoolEntry::Utf8(s) = e {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow!("Invalid name in NameAndType"))?;

        let descriptor = self
            .constant_pool
            .get(descriptor_index as usize)
            .and_then(|e| e.as_ref())
            .and_then(|e| {
                if let ConstantPoolEntry::Utf8(s) = e {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow!("Invalid descriptor in NameAndType"))?;

        Ok((name, descriptor))
    }

    pub fn resolve_class_ref(&mut self, index: u16) -> Result<String> {
        // 1. 先检查缓存
        if let Some(class_name) = self.runtime_pool.resolved_classes.get(&index) {
            return Ok(class_name.clone()); // 🚀 缓存命中
        }

        // 2. 缓存未命中，解析常量池
        let class_entry = self
            .constant_pool
            .get(index as usize)
            .ok_or_else(|| anyhow!("Invalid class index: {}", index))?
            .as_ref()
            .ok_or_else(|| anyhow!("Class entry is None"))?;

        let class_name = if let ConstantPoolEntry::Class { name_index } = class_entry {
            let name_entry = self
                .constant_pool
                .get(*name_index as usize)
                .ok_or_else(|| anyhow!("Invalid name index: {}", name_index))?
                .as_ref()
                .ok_or_else(|| anyhow!("Name entry is None"))?;

            if let ConstantPoolEntry::Utf8(name) = name_entry {
                name.clone()
            } else {
                return Err(anyhow!("Expected Utf8 for class name"));
            }
        } else {
            return Err(anyhow!("Expected Class entry"));
        };

        // 3. 存入缓存
        self.runtime_pool
            .resolved_classes
            .insert(index, class_name.clone());

        Ok(class_name)
    }

    /// 解析方法引用（从常量池索引到方法元数据）
    pub fn resolve_method_ref(
        &mut self,
        index: u16,
    ) -> Result<ResolvedMethodRef> {
        // 先检查缓存
        if let Some(resolved) = self.runtime_pool.resolved_methods.get(&index) {
            return Ok(resolved.clone());
        }

        // 从常量池解析
        let cp_entry = self
            .constant_pool
            .get(index as usize)
            .ok_or_else(|| anyhow!("Invalid constant pool index: {}", index))?
            .as_ref()
            .ok_or_else(|| anyhow!("Constant pool entry is None at index: {}", index))?;

        let (class_index, name_and_type_index) = match cp_entry {
            ConstantPoolEntry::MethodRef {
                class_index,
                name_and_type_index,
            } => (*class_index, *name_and_type_index),
            ConstantPoolEntry::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => (*class_index, *name_and_type_index),
            _ => {
                return Err(anyhow!(
                    "Expected MethodRef or InterfaceMethodRef at index {}",
                    index
                ))
            }
        };

        // 复用 resolve_class_ref 解析类名
        let class_name = self.resolve_class_ref(class_index)?;

        // 复用 resolve_name_and_type 解析方法名和描述符
        let (method_name, descriptor) = self.resolve_name_and_type(name_and_type_index)?;

        // 创建解析结果
        let resolved = ResolvedMethodRef {
            class_name,
            method_name,
            descriptor,
        };

        // 缓存解析结果
        self.runtime_pool
            .resolved_methods
            .insert(index, resolved.clone());

        Ok(resolved)
    }

    /// 解析字段引用
    pub fn resolve_field_ref(
        &mut self,
        index: u16,
    ) -> Result<ResolvedFieldRef> {
        // 先检查缓存
        if let Some(resolved) = self.runtime_pool.resolved_fields.get(&index) {
            return Ok(resolved.clone());
        }

        // 从常量池解析
        let cp_entry = self
            .constant_pool
            .get(index as usize)
            .ok_or_else(|| anyhow!("Invalid constant pool index: {}", index))?
            .as_ref()
            .ok_or_else(|| anyhow!("Constant pool entry is None"))?;

        let (class_index, name_and_type_index) = match cp_entry {
            ConstantPoolEntry::FieldRef {
                class_index,
                name_and_type_index,
            } => (*class_index, *name_and_type_index),
            _ => return Err(anyhow!("Expected FieldRef at index {}", index)),
        };

        // 复用 resolve_class_ref 解析类名
        let class_name = self.resolve_class_ref(class_index)?;

        // 复用 resolve_name_and_type 解析字段名和描述符
        let (field_name, descriptor) = self.resolve_name_and_type(name_and_type_index)?;

        // 创建解析结果
        let resolved = ResolvedFieldRef {
            class_name,
            field_name,
            descriptor,
        };

        // 缓存解析结果
        self.runtime_pool
            .resolved_fields
            .insert(index, resolved.clone());

        Ok(resolved)
    }
}

impl RuntimeConstantPool {
    /// 创建新的运行时常量池
    pub fn new() -> Self {
        RuntimeConstantPool {
            resolved_methods: HashMap::new(),
            resolved_fields: HashMap::new(),
            resolved_classes: HashMap::new(),
        }
    }
}

impl Default for Metaspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RuntimeConstantPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metaspace_creation() {
        let metaspace = Metaspace::new();
        assert_eq!(metaspace.loaded_classes().len(), 0);
    }

    #[test]
    fn test_class_state() {
        let state = ClassState::Loaded;
        assert_eq!(state, ClassState::Loaded);
        assert_ne!(state, ClassState::Initialized);
    }

    #[test]
    fn test_load_class() -> Result<()> {
        let mut metaspace = Metaspace::new();

        // 加载 ReturnOne.class
        let class_file = ClassFile::from_file("examples/ReturnOne.class")?;
        metaspace.load_class(class_file)?;

        // 验证类已加载
        assert!(metaspace.is_class_loaded("ReturnOne"));

        // 获取类元数据
        let class_meta = metaspace.get_class("ReturnOne")?;
        assert_eq!(class_meta.name, "ReturnOne");
        assert_eq!(class_meta.state, ClassState::Loaded);

        Ok(())
    }

    #[test]
    fn test_find_method() -> Result<()> {
        let mut metaspace = Metaspace::new();

        // 加载类
        let class_file = ClassFile::from_file("examples/ReturnOne.class")?;
        metaspace.load_class(class_file)?;

        // 获取类元数据
        let class_meta = metaspace.get_class("ReturnOne")?;

        // 查找方法
        let method = class_meta.find_method("returnOne", "()I")?;
        assert_eq!(method.name, "returnOne");
        assert_eq!(method.descriptor, "()I");
        assert!(method.is_static);
        assert!(!method.is_native);
        assert_eq!(method.max_stack, 1);
        assert_eq!(method.max_locals, 0);
        assert!(!method.code.is_empty());

        Ok(())
    }

    #[test]
    fn test_method_metadata() -> Result<()> {
        let mut metaspace = Metaspace::new();

        let class_file = ClassFile::from_file("examples/ReturnOne.class")?;
        metaspace.load_class(class_file)?;

        let class_meta = metaspace.get_class("ReturnOne")?;

        // ReturnOne 应该有多个方法（包括<init>）
        assert!(class_meta.methods.len() > 0);

        Ok(())
    }

    #[test]
    fn test_class_hierarchy() -> Result<()> {
        let mut metaspace = Metaspace::new();

        let class_file = ClassFile::from_file("examples/ReturnOne.class")?;
        metaspace.load_class(class_file)?;

        let class_meta = metaspace.get_class("ReturnOne")?;

        // 所有类都应该有父类（除了Object）
        assert!(class_meta.super_class.is_some());
        assert_eq!(class_meta.super_class.as_ref().unwrap(), "java/lang/Object");

        Ok(())
    }

    #[test]
    fn test_runtime_constant_pool() {
        let runtime_pool = RuntimeConstantPool::new();
        assert_eq!(runtime_pool.resolved_methods.len(), 0);
        assert_eq!(runtime_pool.resolved_fields.len(), 0);
    }

    #[test]
    fn test_multiple_classes() -> Result<()> {
        let mut metaspace = Metaspace::new();

        // 加载多个类
        let class1 = ClassFile::from_file("examples/ReturnOne.class")?;
        metaspace.load_class(class1)?;

        let class2 = ClassFile::from_file("examples/Calculator.class")?;
        metaspace.load_class(class2)?;

        // 验证两个类都已加载
        assert_eq!(metaspace.loaded_classes().len(), 2);
        assert!(metaspace.is_class_loaded("ReturnOne"));
        assert!(metaspace.is_class_loaded("Calculator"));

        Ok(())
    }

    #[test]
    fn test_duplicate_class_load() -> Result<()> {
        let mut metaspace = Metaspace::new();

        // 第一次加载
        let class_file = ClassFile::from_file("examples/ReturnOne.class")?;
        metaspace.load_class(class_file)?;

        // 第二次加载同一个类（应该被忽略）
        let class_file = ClassFile::from_file("examples/ReturnOne.class")?;
        metaspace.load_class(class_file)?;

        // 应该只有一个类
        assert_eq!(metaspace.loaded_classes().len(), 1);

        Ok(())
    }
}
