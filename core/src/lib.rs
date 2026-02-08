//! Asset Manager Core Library
//! 
//! 提供资产管理的核心功能：
//! - 资产模型定义
//! - Lua 插件系统
//! - JSON 本地存储

pub mod asset;
pub mod plugin;
pub mod storage;

pub use asset::*;
pub use plugin::PluginManager;
pub use storage::Database;

/// 应用程序配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    /// 数据文件路径
    pub db_path: String,
    /// 插件目录路径
    pub plugins_dir: String,
    /// 是否启用调试模式
    pub debug: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            db_path: "data/assets.json".to_string(),
            plugins_dir: "plugins".to_string(),
            debug: false,
        }
    }
}
