//! 插件管理器

use super::{PluginError, PluginEvent, PluginInfo, PluginLoader};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

/// 插件管理器
pub struct PluginManager {
    /// 插件目录
    plugins_dir: PathBuf,
    /// 已加载的插件
    plugins: HashMap<String, (PluginInfo, PluginLoader)>,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new(plugins_dir: impl Into<PathBuf>) -> Self {
        Self {
            plugins_dir: plugins_dir.into(),
            plugins: HashMap::new(),
        }
    }

    /// 扫描并加载所有插件
    pub fn load_all(&mut self) -> Result<Vec<PluginInfo>, PluginError> {
        let mut loaded = Vec::new();

        if !self.plugins_dir.exists() {
            info!("Creating plugins directory: {:?}", self.plugins_dir);
            fs::create_dir_all(&self.plugins_dir)?;
            return Ok(loaded);
        }

        for entry in fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                match self.load_plugin(&path) {
                    Ok(info) => {
                        info!("Loaded plugin: {} v{}", info.name, info.version);
                        loaded.push(info);
                    }
                    Err(e) => {
                        warn!("Failed to load plugin from {:?}: {}", path, e);
                    }
                }
            }
        }

        info!("Loaded {} plugins", loaded.len());
        Ok(loaded)
    }

    /// 加载单个插件
    pub fn load_plugin(&mut self, plugin_dir: &Path) -> Result<PluginInfo, PluginError> {
        let loader = PluginLoader::new()?;
        let info = loader.load_from_dir(plugin_dir)?;

        // 调用插件的 on_load 函数（如果存在）
        if let Err(e) = self.call_plugin_lifecycle(&loader, "on_load", ()) {
            warn!("Plugin {} on_load error: {}", info.name, e);
        }

        let name = info.name.clone();
        self.plugins.insert(name, (info.clone(), loader));

        Ok(info)
    }

    /// 卸载插件
    pub fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        if let Some((info, loader)) = self.plugins.remove(name) {
            // 调用 on_unload
            let _ = self.call_plugin_lifecycle(&loader, "on_unload", ());
            info!("Unloaded plugin: {}", info.name);
            Ok(())
        } else {
            Err(PluginError::NotFound(name.to_string()))
        }
    }

    /// 获取已加载的插件列表
    pub fn list_plugins(&self) -> Vec<&PluginInfo> {
        self.plugins.values().map(|(info, _)| info).collect()
    }

    /// 获取插件信息
    pub fn get_plugin(&self, name: &str) -> Option<&PluginInfo> {
        self.plugins.get(name).map(|(info, _)| info)
    }

    /// 启用/禁用插件
    pub fn set_plugin_enabled(&mut self, name: &str, enabled: bool) -> Result<(), PluginError> {
        if let Some((info, _)) = self.plugins.get_mut(name) {
            info.enabled = enabled;
            info!(
                "Plugin {} {}",
                name,
                if enabled { "enabled" } else { "disabled" }
            );
            Ok(())
        } else {
            Err(PluginError::NotFound(name.to_string()))
        }
    }

    /// 广播事件到所有插件
    pub fn broadcast_event(&self, event: &PluginEvent) {
        for (info, loader) in self.plugins.values() {
            if !info.enabled {
                continue;
            }

            let result = match event {
                PluginEvent::AssetCreated(asset) => {
                    self.call_plugin_with_json(loader, "on_asset_created", asset)
                }
                PluginEvent::AssetUpdated(asset) => {
                    self.call_plugin_with_json(loader, "on_asset_updated", asset)
                }
                PluginEvent::AssetDeleted(id) => {
                    self.call_plugin_with_json(loader, "on_asset_deleted", &id.to_string())
                }
                PluginEvent::AppStarted => {
                    self.call_plugin_lifecycle(loader, "on_app_started", ())
                }
                PluginEvent::AppClosing => {
                    self.call_plugin_lifecycle(loader, "on_app_closing", ())
                }
                PluginEvent::Custom(event_name, data) => {
                    self.call_plugin_custom(loader, event_name, data)
                }
            };

            if let Err(e) = result {
                // 只在函数存在但执行失败时警告
                if !matches!(e, PluginError::NotFound(_)) {
                    error!("Plugin {} event error: {}", info.name, e);
                }
            }
        }
    }

    /// 调用插件生命周期函数
    fn call_plugin_lifecycle<A>(
        &self,
        loader: &PluginLoader,
        func_name: &str,
        args: A,
    ) -> Result<(), PluginError>
    where
        A: mlua::IntoLuaMulti + Clone,
    {
        loader.call_function::<_, ()>(func_name, args)
    }

    /// 调用插件函数并传递 JSON 数据
    fn call_plugin_with_json<T: serde::Serialize>(
        &self,
        loader: &PluginLoader,
        func_name: &str,
        data: &T,
    ) -> Result<(), PluginError> {
        let json_str = serde_json::to_string(data).unwrap_or_default();
        loader.call_function::<_, ()>(func_name, json_str)
    }

    /// 调用自定义事件
    fn call_plugin_custom(
        &self,
        loader: &PluginLoader,
        event_name: &str,
        data: &serde_json::Value,
    ) -> Result<(), PluginError> {
        let handler_name = format!("on_{}", event_name);
        let json_str = serde_json::to_string(data).unwrap_or_default();
        loader.call_function::<_, ()>(&handler_name, json_str)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new("plugins")
    }
}
