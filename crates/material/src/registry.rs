//! 物料注册表：按 id 查找物料，管理内置物料与用户物料。
//!
//! 占位文件：完整实现在 Step 4 完成。

use crate::error::MaterialResult;
use crate::material::{Material, MaterialId, MaterialInfo};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// 物料注册表，按 id 查找物料。
pub struct MaterialRegistry {
    materials: HashMap<MaterialId, Arc<Material>>,
}

impl MaterialRegistry {
    /// 创建空注册表。
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
        }
    }

    /// 加载全部内置物料（通过 `include_str!` 嵌入）。
    pub fn load_builtin(&mut self) -> MaterialResult<()> {
        for (name, source) in crate::BUILTIN_MATERIALS {
            let id = format!("{}{}", crate::BUILTIN_PREFIX, name);
            let material = Material::load(id.clone(), source, true)?;
            self.materials.insert(id, Arc::new(material));
        }
        Ok(())
    }

    /// 扫描用户物料目录并加载 `.rhai` 文件。
    ///
    /// 文件名（不含扩展名）作为物料名称，id 形如 `user.<name>`。
    /// 同名时用户物料覆盖内置物料。
    pub fn load_user(&mut self, dir: &Path) -> MaterialResult<()> {
        if !dir.exists() {
            return Ok(());
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                return Err(crate::MaterialError::DirScan {
                    dir: dir.display().to_string(),
                    source: e,
                });
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rhai") {
                continue;
            }
            let name = match path.file_stem().and_then(|s| s.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let source = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(name = %name, error = %e, "failed to read user material");
                    continue;
                }
            };
            // 用户物料覆盖同名内置物料：user.<name> 与 builtin.<name> 共存，
            // 但 evaluate 时优先 user。
            let user_id = format!("{}{}", crate::USER_PREFIX, name);
            match Material::load(user_id.clone(), &source, false) {
                Ok(m) => {
                    self.materials.insert(user_id.clone(), Arc::new(m));
                    tracing::info!(id = %user_id, "loaded user material");
                }
                Err(e) => {
                    tracing::warn!(id = %user_id, error = %e, "failed to load user material");
                }
            }
        }
        Ok(())
    }

    /// 按 id 查找物料。
    pub fn get(&self, id: &str) -> Option<Arc<Material>> {
        self.materials.get(id).cloned()
    }

    /// 列出全部物料信息。
    pub fn list(&self) -> Vec<MaterialInfo> {
        self.materials
            .values()
            .map(|m| {
                let builtin = m.metadata().id.starts_with(crate::BUILTIN_PREFIX);
                m.info(builtin)
            })
            .collect()
    }

    /// 已注册的物料数量。
    pub fn len(&self) -> usize {
        self.materials.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }
}

impl Default for MaterialRegistry {
    fn default() -> Self {
        Self::new()
    }
}
