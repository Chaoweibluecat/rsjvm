//! # 类加载器
//!
//! 类加载器负责将class文件加载到JVM中。
//!
//! ## 学习要点
//! - 类加载过程：加载 -> 验证 -> 准备 -> 解析 -> 初始化
//! - 双亲委派模型
//! - 类的生命周期
//!
//! ## 简化设计
//! 这个实现简化了类加载过程，主要关注加载和基本验证

use crate::classfile::ClassFile;
use crate::Result;
use anyhow::{anyhow, Context};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 类加载器
pub struct ClassLoader {
    /// 类路径
    class_paths: Vec<PathBuf>,
    /// 已加载的类
    loaded_classes: HashMap<String, ClassFile>,
}

impl ClassLoader {
    /// 创建新的类加载器
    pub fn new(class_paths: Vec<PathBuf>) -> Self {
        ClassLoader {
            class_paths,
            loaded_classes: HashMap::new(),
        }
    }

    /// 加载类
    pub fn load_class(&mut self, class_name: &str) -> Result<&ClassFile> {
        // 检查是否已加载
        if self.loaded_classes.contains_key(class_name) {
            return Ok(&self.loaded_classes[class_name]);
        }

        // 将类名转换为文件路径（例如：java/lang/Object -> java/lang/Object.class）
        let class_file_name = format!("{}.class", class_name);

        // 在类路径中搜索
        for class_path in &self.class_paths {
            let class_file_path = class_path.join(&class_file_name);
            if class_file_path.exists() {
                let class_file = ClassFile::from_file(&class_file_path)
                    .context(format!("Failed to load class: {}", class_name))?;

                // 验证类名是否匹配
                let loaded_name = class_file.get_class_name()?;
                if loaded_name != class_name {
                    return Err(anyhow!(
                        "Class name mismatch: expected {}, got {}",
                        class_name,
                        loaded_name
                    ));
                }

                self.loaded_classes
                    .insert(class_name.to_string(), class_file);
                return Ok(&self.loaded_classes[class_name]);
            }
        }

        Err(anyhow!("Class not found: {}", class_name))
    }

    /// 获取已加载的类
    pub fn get_loaded_class(&self, class_name: &str) -> Option<&ClassFile> {
        self.loaded_classes.get(class_name)
    }

    /// 添加类路径
    pub fn add_class_path<P: AsRef<Path>>(&mut self, path: P) {
        self.class_paths.push(path.as_ref().to_path_buf());
    }
}
