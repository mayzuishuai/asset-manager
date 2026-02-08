//! Lua 插件系统

mod loader;
mod manager;

pub use loader::PluginLoader;
pub use manager::PluginManager;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 插件元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// 插件名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 作者
    pub author: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 插件目录路径
    pub path: PathBuf,
    /// 是否启用
    pub enabled: bool,
}

/// 插件事件
#[derive(Debug, Clone)]
pub enum PluginEvent {
    /// 资产创建
    AssetCreated(crate::Asset),
    /// 资产更新
    AssetUpdated(crate::Asset),
    /// 资产删除
    AssetDeleted(uuid::Uuid),
    /// 应用启动
    AppStarted,
    /// 应用关闭
    AppClosing,
    /// 自定义事件
    Custom(String, serde_json::Value),
}

/// 插件错误
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Plugin load error: {0}")]
    LoadError(String),
    
    #[error("Lua error: {0}")]
    LuaError(#[from] mlua::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Plugin disabled: {0}")]
    Disabled(String),
}
