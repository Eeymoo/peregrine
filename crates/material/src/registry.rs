//! 物料注册表：按 id 查找物料，管理内置物料与用户物料。
//!
//! 内部使用 `RwLock<HashMap>`，支持运行时热重载（用户物料目录变化时整体替换）。

use crate::error::MaterialResult;
use crate::material::{Material, MaterialId, MaterialInfo};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

/// 物料注册表，按 id 查找物料。
///
/// 内部使用 `RwLock`，支持多线程读 + 单线程写（热重载）。
/// `Arc<MaterialRegistry>` 可被多个组件共享，无需外部锁。
#[derive(Clone)]
pub struct MaterialRegistry {
    inner: Arc<RwLock<HashMap<MaterialId, Arc<Material>>>>,
}

impl MaterialRegistry {
    /// 创建空注册表。
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 加载全部内置物料（通过 `include_str!` 嵌入）。
    ///
    /// 写锁，会阻塞读操作。内置物料加载只应在启动时调用一次。
    pub fn load_builtin(&self) -> MaterialResult<()> {
        let mut map = self.inner.write().expect("MaterialRegistry poisoned");
        for (name, source) in crate::BUILTIN_MATERIALS {
            let id = format!("{}{}", crate::BUILTIN_PREFIX, name);
            let material = Material::load(id.clone(), source, true)?;
            map.insert(id, Arc::new(material));
        }
        Ok(())
    }

    /// 扫描用户物料目录并加载 `.rhai` 文件。
    ///
    /// 文件名（不含扩展名）作为物料名称，id 形如 `user.<name>`。
    /// 同名时用户物料覆盖内置物料。
    ///
    /// 此方法可安全地在运行时多次调用（用于热重载）。
    pub fn load_user(&self, dir: &Path) -> MaterialResult<()> {
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

        // 先收集本次扫描要加入的 user.* 物料，再批量写入，减少锁竞争。
        let mut new_entries: HashMap<MaterialId, Arc<Material>> = HashMap::new();
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
            let user_id = format!("{}{}", crate::USER_PREFIX, name);
            match Material::load(user_id.clone(), &source, false) {
                Ok(m) => {
                    new_entries.insert(user_id.clone(), Arc::new(m));
                    tracing::info!(id = %user_id, "loaded user material");
                }
                Err(e) => {
                    tracing::warn!(id = %user_id, error = %e, "failed to load user material");
                }
            }
        }

        // 批量替换：先删除所有旧 user.* 物料，再插入新物料。
        let mut map = self.inner.write().expect("MaterialRegistry poisoned");
        // 移除所有旧用户物料。
        let user_keys: Vec<String> = map
            .keys()
            .filter(|k| k.starts_with(crate::USER_PREFIX))
            .cloned()
            .collect();
        for k in user_keys {
            map.remove(&k);
        }
        // 插入新物料。
        for (k, v) in new_entries {
            map.insert(k, v);
        }
        Ok(())
    }

    /// 按 id 查找物料（返回 `Arc<Material>`，可在锁外长期持有）。
    pub fn get(&self, id: &str) -> Option<Arc<Material>> {
        let map = self.inner.read().expect("MaterialRegistry poisoned");
        map.get(id).cloned()
    }

    /// 列出全部物料信息。
    pub fn list(&self) -> Vec<MaterialInfo> {
        let map = self.inner.read().expect("MaterialRegistry poisoned");
        map.values()
            .map(|m| {
                let builtin = m.metadata().id.starts_with(crate::BUILTIN_PREFIX);
                m.info(builtin)
            })
            .collect()
    }

    /// 已注册的物料数量。
    pub fn len(&self) -> usize {
        let map = self.inner.read().expect("MaterialRegistry poisoned");
        map.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for MaterialRegistry {
    fn default() -> Self {
        Self::new()
    }
}
